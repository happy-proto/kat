use std::{io::Cursor, path::Path};

use anyhow::{Context, Result, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use icy_sixel::SixelImage;
use image::{
    DynamicImage, GenericImageView, ImageDecoder, ImageFormat as EncodedImageFormat, ImageReader,
    RgbaImage, imageops::FilterType, metadata::Orientation,
};
use serde::Serialize;
use termwiz::{
    caps::Capabilities,
    escape::osc::{ITermDimension, ITermFileData, ITermProprietary, OperatingSystemCommand},
};

const KITTY_CHUNK_BYTES: usize = 3072;
const DEFAULT_MAX_HEIGHT_RATIO_NUMERATOR: usize = 4;
const DEFAULT_MAX_HEIGHT_RATIO_DENOMINATOR: usize = 5;
const APPROX_CELL_PIXEL_WIDTH: usize = 8;
const APPROX_CELL_PIXEL_HEIGHT: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ImageFormat {
    Bmp,
    Dds,
    Farbfeld,
    Gif,
    Hdr,
    Ico,
    Jpeg,
    Pnm,
    Png,
    Qoi,
    Tiff,
    Webp,
}

impl ImageFormat {
    fn as_str(self) -> &'static str {
        match self {
            Self::Bmp => "bmp",
            Self::Dds => "dds",
            Self::Farbfeld => "farbfeld",
            Self::Gif => "gif",
            Self::Hdr => "hdr",
            Self::Ico => "ico",
            Self::Jpeg => "jpeg",
            Self::Pnm => "pnm",
            Self::Png => "png",
            Self::Qoi => "qoi",
            Self::Tiff => "tiff",
            Self::Webp => "webp",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ImageFit {
    Contain,
    Original,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ImageBackground {
    Terminal,
    Black,
    White,
    Checker,
}

impl ImageBackground {
    fn as_str(self) -> &'static str {
        match self {
            Self::Terminal => "terminal",
            Self::Black => "black",
            Self::White => "white",
            Self::Checker => "checker",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ImageRenderOptions {
    pub(crate) width_cells: Option<usize>,
    pub(crate) height_cells: Option<usize>,
    pub(crate) fit: ImageFit,
    pub(crate) background: ImageBackground,
    pub(crate) terminal_columns: Option<usize>,
    pub(crate) terminal_rows: Option<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ImageCellSize {
    columns: usize,
    rows: usize,
}

pub(crate) struct TerminalImageOutput {
    pub(crate) output: String,
    pub(crate) contains_terminal_image: bool,
}

pub(crate) fn sniff_image_format(bytes: &[u8]) -> Option<ImageFormat> {
    if is_bmp_file_header(bytes) {
        return Some(ImageFormat::Bmp);
    }

    if is_ico_file_header(bytes) {
        return Some(ImageFormat::Ico);
    }

    if is_pnm_file_header(bytes) {
        return Some(ImageFormat::Pnm);
    }

    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some(ImageFormat::Png);
    }

    if bytes.starts_with(b"\xff\xd8\xff") {
        return Some(ImageFormat::Jpeg);
    }

    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some(ImageFormat::Gif);
    }

    if bytes.starts_with(b"II*\0")
        || bytes.starts_with(b"MM\0*")
        || bytes.starts_with(b"II+\0")
        || bytes.starts_with(b"MM\0+")
    {
        return Some(ImageFormat::Tiff);
    }

    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some(ImageFormat::Webp);
    }

    if bytes.starts_with(b"qoif") {
        return Some(ImageFormat::Qoi);
    }

    if bytes.starts_with(b"farbfeld") {
        return Some(ImageFormat::Farbfeld);
    }

    if bytes.starts_with(b"#?RADIANCE") {
        return Some(ImageFormat::Hdr);
    }

    if bytes.starts_with(b"DDS ") {
        return Some(ImageFormat::Dds);
    }

    None
}

fn is_bmp_file_header(bytes: &[u8]) -> bool {
    if bytes.len() < 18 || !bytes.starts_with(b"BM") {
        return false;
    }

    let reserved = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]);
    let dib_header_size = u32::from_le_bytes([bytes[14], bytes[15], bytes[16], bytes[17]]);
    reserved == 0 && matches!(dib_header_size, 12 | 40 | 52 | 56 | 108 | 124)
}

fn is_ico_file_header(bytes: &[u8]) -> bool {
    if bytes.len() < 6 || &bytes[..4] != b"\0\0\x01\0" {
        return false;
    }

    let image_count = u16::from_le_bytes([bytes[4], bytes[5]]);
    image_count > 0
}

fn is_pnm_file_header(bytes: &[u8]) -> bool {
    bytes.len() >= 3
        && bytes[0] == b'P'
        && matches!(bytes[1], b'1'..=b'7')
        && bytes[2].is_ascii_whitespace()
}

pub(crate) fn render_inline_image(
    path: &Path,
    bytes: Vec<u8>,
    stdout_is_terminal: bool,
    options: ImageRenderOptions,
) -> Result<TerminalImageOutput> {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned());
    let analysis = analyze_image(path, &bytes, stdout_is_terminal, options)?;

    if !stdout_is_terminal {
        return Ok(TerminalImageOutput {
            output: image_info_text(&analysis, Some("stdout is not a TTY")),
            contains_terminal_image: false,
        });
    }

    match select_image_protocol()? {
        Some(ImageProtocol::Iterm2) => Ok(TerminalImageOutput {
            output: encode_iterm2_image(name, bytes, &analysis)?,
            contains_terminal_image: true,
        }),
        Some(ImageProtocol::Kitty) => Ok(TerminalImageOutput {
            output: encode_kitty_image(&analysis)?,
            contains_terminal_image: true,
        }),
        Some(ImageProtocol::Sixel) => Ok(TerminalImageOutput {
            output: encode_sixel_image(&analysis)?,
            contains_terminal_image: true,
        }),
        None => Ok(TerminalImageOutput {
            output: image_info_text(
                &analysis,
                Some("terminal image protocol unsupported or disabled"),
            ),
            contains_terminal_image: false,
        }),
    }
}

pub(crate) fn debug_image_json(
    path: &Path,
    bytes: &[u8],
    stdout_is_terminal: bool,
    options: ImageRenderOptions,
) -> Result<String> {
    let analysis = analyze_image(path, bytes, stdout_is_terminal, options)?;
    Ok(serde_json::to_string_pretty(&analysis.snapshot())?)
}

fn supports_iterm2_inline_images() -> bool {
    Capabilities::new_from_env()
        .map(|capabilities| capabilities.iterm2_image())
        .unwrap_or(false)
        || env_var_contains("TERM_PROGRAM", "rio")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ImageProtocol {
    Iterm2,
    Kitty,
    Sixel,
}

impl ImageProtocol {
    fn as_str(self) -> &'static str {
        match self {
            Self::Iterm2 => "iterm2",
            Self::Kitty => "kitty",
            Self::Sixel => "sixel",
        }
    }
}

fn select_image_protocol() -> Result<Option<ImageProtocol>> {
    if let Some(protocol) = forced_image_protocol()? {
        return Ok(protocol);
    }

    if supports_iterm2_inline_images() {
        return Ok(Some(ImageProtocol::Iterm2));
    }

    if supports_kitty_graphics() {
        return Ok(Some(ImageProtocol::Kitty));
    }

    if supports_sixel_graphics() {
        return Ok(Some(ImageProtocol::Sixel));
    }

    Ok(None)
}

fn forced_image_protocol() -> Result<Option<Option<ImageProtocol>>> {
    let Some(value) = std::env::var_os("KAT_IMAGE_PROTOCOL") else {
        return Ok(None);
    };
    let value = value.to_string_lossy().trim().to_ascii_lowercase();
    let protocol = match value.as_str() {
        "" | "auto" => None,
        "none" | "off" | "never" => Some(None),
        "iterm" | "iterm2" => Some(Some(ImageProtocol::Iterm2)),
        "kitty" => Some(Some(ImageProtocol::Kitty)),
        "sixel" => Some(Some(ImageProtocol::Sixel)),
        other => bail!("unsupported KAT_IMAGE_PROTOCOL value `{other}`"),
    };
    Ok(protocol)
}

fn supports_kitty_graphics() -> bool {
    std::env::var_os("KITTY_WINDOW_ID").is_some()
        || std::env::var_os("KONSOLE_VERSION").is_some()
        || env_var_contains("TERM", "kitty")
        || env_var_contains("TERM_PROGRAM", "kitty")
        || env_var_contains("TERM_PROGRAM", "ghostty")
}

fn supports_sixel_graphics() -> bool {
    Capabilities::new_from_env()
        .map(|capabilities| capabilities.sixel())
        .unwrap_or(false)
        || env_var_contains("TERM", "sixel")
        || env_var_contains("TERM", "foot")
        || env_var_contains("TERM_PROGRAM", "foot")
        || env_var_contains("TERM_PROGRAM", "mlterm")
        || env_var_contains("TERM_PROGRAM", "mintty")
}

fn env_var_contains(name: &str, needle: &str) -> bool {
    std::env::var(name)
        .map(|value| value.to_ascii_lowercase().contains(needle))
        .unwrap_or(false)
}

#[derive(Clone, Debug)]
struct ImageAnalysis {
    path: String,
    format: ImageFormat,
    input_bytes: usize,
    original_width: usize,
    original_height: usize,
    display_width: usize,
    display_height: usize,
    orientation: Orientation,
    cell_size: Option<ImageCellSize>,
    protocol: Option<ImageProtocol>,
    stdout_is_terminal: bool,
    options: ImageRenderOptions,
    decoded: DynamicImage,
}

impl ImageAnalysis {
    fn snapshot(&self) -> ImageDebugSnapshot {
        ImageDebugSnapshot {
            path: self.path.clone(),
            format: self.format.as_str(),
            input_bytes: self.input_bytes,
            original_width: self.original_width,
            original_height: self.original_height,
            display_width: self.display_width,
            display_height: self.display_height,
            orientation: orientation_name(self.orientation),
            orientation_applied: self.orientation != Orientation::NoTransforms,
            target_cells: self.cell_size.map(ImageCellSizeSnapshot::from),
            protocol: self.protocol.map(ImageProtocol::as_str),
            stdout_is_terminal: self.stdout_is_terminal,
            fit: match self.options.fit {
                ImageFit::Contain => "contain",
                ImageFit::Original => "original",
            },
            background: self.options.background.as_str(),
            frame_note: frame_note(self.format),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
struct ImageCellSizeSnapshot {
    columns: usize,
    rows: usize,
}

impl From<ImageCellSize> for ImageCellSizeSnapshot {
    fn from(value: ImageCellSize) -> Self {
        Self {
            columns: value.columns,
            rows: value.rows,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ImageDebugSnapshot {
    path: String,
    format: &'static str,
    input_bytes: usize,
    original_width: usize,
    original_height: usize,
    display_width: usize,
    display_height: usize,
    orientation: &'static str,
    orientation_applied: bool,
    target_cells: Option<ImageCellSizeSnapshot>,
    protocol: Option<&'static str>,
    stdout_is_terminal: bool,
    fit: &'static str,
    background: &'static str,
    frame_note: Option<&'static str>,
}

fn analyze_image(
    path: &Path,
    bytes: &[u8],
    stdout_is_terminal: bool,
    options: ImageRenderOptions,
) -> Result<ImageAnalysis> {
    let format = sniff_image_format(bytes).expect("caller should only pass known image formats");
    let decoded = decode_dynamic_image(bytes)?;
    let (display_width, display_height) = decoded.image.dimensions();
    let cell_size = resolve_cell_size(display_width as usize, display_height as usize, options);

    Ok(ImageAnalysis {
        path: path.display().to_string(),
        format,
        input_bytes: bytes.len(),
        original_width: decoded.original_width,
        original_height: decoded.original_height,
        display_width: display_width as usize,
        display_height: display_height as usize,
        orientation: decoded.orientation,
        cell_size,
        protocol: select_image_protocol()?,
        stdout_is_terminal,
        options,
        decoded: decoded.image,
    })
}

struct DecodedDynamicImage {
    image: DynamicImage,
    original_width: usize,
    original_height: usize,
    orientation: Orientation,
}

fn decode_dynamic_image(data: &[u8]) -> Result<DecodedDynamicImage> {
    let reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .context("failed to guess image format")?;
    let mut decoder = reader
        .into_decoder()
        .context("failed to create image decoder")?;
    let (original_width, original_height) = decoder.dimensions();
    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let mut image = DynamicImage::from_decoder(decoder).context("failed to decode image")?;
    image.apply_orientation(orientation);

    Ok(DecodedDynamicImage {
        image,
        original_width: original_width as usize,
        original_height: original_height as usize,
        orientation,
    })
}

fn image_info_text(analysis: &ImageAnalysis, reason: Option<&str>) -> String {
    let mut output = format!(
        "{}: {} {}x{}, {}",
        analysis.path,
        analysis.format.as_str().to_ascii_uppercase(),
        analysis.display_width,
        analysis.display_height,
        format_bytes(analysis.input_bytes),
    );
    if analysis.orientation != Orientation::NoTransforms {
        output.push_str(&format!(
            ", orientation {} applied",
            orientation_name(analysis.orientation)
        ));
    }
    if let Some(size) = analysis.cell_size {
        output.push_str(&format!(", target {}x{} cells", size.columns, size.rows));
    }
    output.push('\n');
    if let Some(note) = frame_note(analysis.format) {
        output.push_str(note);
        output.push('\n');
    }
    if let Some(reason) = reason {
        output.push_str(reason);
        output.push('\n');
    }
    output
}

fn format_bytes(bytes: usize) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    if bytes as f64 >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB)
    } else if bytes as f64 >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB)
    } else {
        format!("{bytes} B")
    }
}

fn orientation_name(orientation: Orientation) -> &'static str {
    match orientation {
        Orientation::NoTransforms => "none",
        Orientation::Rotate90 => "rotate90",
        Orientation::Rotate180 => "rotate180",
        Orientation::Rotate270 => "rotate270",
        Orientation::FlipHorizontal => "flip_horizontal",
        Orientation::FlipVertical => "flip_vertical",
        Orientation::Rotate90FlipH => "rotate90_flip_horizontal",
        Orientation::Rotate270FlipH => "rotate270_flip_horizontal",
    }
}

fn frame_note(format: ImageFormat) -> Option<&'static str> {
    match format {
        ImageFormat::Gif => Some(
            "animated GIF inputs are rendered as their first frame outside the raw iTerm2 path",
        ),
        ImageFormat::Tiff => Some("multi-page TIFF inputs are rendered as their first page"),
        ImageFormat::Bmp
        | ImageFormat::Dds
        | ImageFormat::Farbfeld
        | ImageFormat::Hdr
        | ImageFormat::Ico
        | ImageFormat::Jpeg
        | ImageFormat::Pnm
        | ImageFormat::Png
        | ImageFormat::Qoi
        | ImageFormat::Webp => None,
    }
}

fn resolve_cell_size(
    image_width: usize,
    image_height: usize,
    options: ImageRenderOptions,
) -> Option<ImageCellSize> {
    if image_width == 0 || image_height == 0 {
        return None;
    }

    let max_width = options.width_cells.or(options.terminal_columns);
    let max_height = options.height_cells.or_else(|| {
        options
            .terminal_rows
            .map(|rows| {
                rows * DEFAULT_MAX_HEIGHT_RATIO_NUMERATOR / DEFAULT_MAX_HEIGHT_RATIO_DENOMINATOR
            })
            .map(|rows| rows.max(1))
    });

    if matches!(options.fit, ImageFit::Original)
        && options.width_cells.is_none()
        && options.height_cells.is_none()
    {
        return None;
    }

    match (max_width, max_height) {
        (Some(width), Some(height)) => Some(contained_cell_size(
            image_width,
            image_height,
            width.max(1),
            height.max(1),
        )),
        (Some(width), None) => {
            let width = width.max(1);
            Some(ImageCellSize {
                columns: width,
                rows: rows_for_width(image_width, image_height, width),
            })
        }
        (None, Some(height)) => {
            let height = height.max(1);
            Some(ImageCellSize {
                columns: columns_for_height(image_width, image_height, height),
                rows: height,
            })
        }
        (None, None) => None,
    }
}

fn contained_cell_size(
    image_width: usize,
    image_height: usize,
    max_width: usize,
    max_height: usize,
) -> ImageCellSize {
    let rows_at_max_width = rows_for_width(image_width, image_height, max_width);
    if rows_at_max_width <= max_height {
        return ImageCellSize {
            columns: max_width,
            rows: rows_at_max_width.max(1),
        };
    }

    ImageCellSize {
        columns: columns_for_height(image_width, image_height, max_height)
            .min(max_width)
            .max(1),
        rows: max_height.max(1),
    }
}

fn rows_for_width(image_width: usize, image_height: usize, columns: usize) -> usize {
    ((columns as f64 * image_height as f64)
        / (image_width as f64 * APPROX_CELL_PIXEL_HEIGHT as f64 / APPROX_CELL_PIXEL_WIDTH as f64))
        .ceil()
        .max(1.0) as usize
}

fn columns_for_height(image_width: usize, image_height: usize, rows: usize) -> usize {
    ((rows as f64 * image_width as f64 * APPROX_CELL_PIXEL_HEIGHT as f64)
        / (image_height as f64 * APPROX_CELL_PIXEL_WIDTH as f64))
        .ceil()
        .max(1.0) as usize
}

fn encode_iterm2_image(
    name: Option<String>,
    original_data: Vec<u8>,
    analysis: &ImageAnalysis,
) -> Result<String> {
    let should_preserve_original = analysis.cell_size.is_none()
        && analysis.orientation == Orientation::NoTransforms
        && analysis.options.background == ImageBackground::Terminal;
    let data = if should_preserve_original {
        original_data
    } else {
        encode_dynamic_png(&rendered_dynamic_image(
            &analysis.decoded,
            analysis.cell_size,
            analysis.options.background,
        )?)?
    };

    Ok(encode_iterm2_inline_image(name, data, analysis.cell_size))
}

fn encode_iterm2_inline_image(
    name: Option<String>,
    data: Vec<u8>,
    cell_size: Option<ImageCellSize>,
) -> String {
    let file = ITermFileData {
        name,
        size: Some(data.len()),
        width: cell_size
            .map(|size| ITermDimension::Cells(size.columns as i64))
            .unwrap_or(ITermDimension::Automatic),
        height: cell_size
            .map(|size| ITermDimension::Cells(size.rows as i64))
            .unwrap_or(ITermDimension::Automatic),
        preserve_aspect_ratio: true,
        inline: true,
        do_not_move_cursor: false,
        data,
    };
    let command = OperatingSystemCommand::ITermProprietary(ITermProprietary::File(Box::new(file)));
    let mut output = command.to_string();
    output.push('\n');
    output
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DecodedRgbaImage {
    pixels: Vec<u8>,
    width: usize,
    height: usize,
}

fn encode_dynamic_png(image: &DynamicImage) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(Vec::new());
    image
        .write_to(&mut cursor, EncodedImageFormat::Png)
        .context("failed to encode image as png")?;
    Ok(cursor.into_inner())
}

fn rendered_dynamic_image(
    image: &DynamicImage,
    cell_size: Option<ImageCellSize>,
    background: ImageBackground,
) -> Result<DynamicImage> {
    let image = resize_to_cell_size(image, cell_size);
    apply_background(image, background)
}

fn decode_rgba_image(
    image: &DynamicImage,
    cell_size: Option<ImageCellSize>,
    background: ImageBackground,
) -> Result<DecodedRgbaImage> {
    let image = resize_to_cell_size(image, cell_size);
    let rgba = rgba_with_background(&image, background)?;
    let (width, height) = rgba.dimensions();

    Ok(DecodedRgbaImage {
        pixels: rgba.into_raw(),
        width: width as usize,
        height: height as usize,
    })
}

fn resize_to_cell_size(image: &DynamicImage, cell_size: Option<ImageCellSize>) -> DynamicImage {
    let Some(size) = cell_size else {
        return image.clone();
    };

    let pixel_width = size.columns.saturating_mul(APPROX_CELL_PIXEL_WIDTH).max(1);
    let pixel_height = size.rows.saturating_mul(APPROX_CELL_PIXEL_HEIGHT).max(1);
    image.resize(
        pixel_width as u32,
        pixel_height as u32,
        FilterType::Triangle,
    )
}

fn rgba_with_background(image: &DynamicImage, background: ImageBackground) -> Result<RgbaImage> {
    let mut rgba = image.to_rgba8();
    if background == ImageBackground::Terminal {
        return Ok(rgba);
    }

    for (x, y, pixel) in rgba.enumerate_pixels_mut() {
        let [red, green, blue, alpha] = pixel.0;
        if alpha == u8::MAX {
            continue;
        }

        let [bg_red, bg_green, bg_blue] = background_rgb(background, x, y);
        let alpha = alpha as u16;
        pixel.0 = [
            blend_channel(red, bg_red, alpha),
            blend_channel(green, bg_green, alpha),
            blend_channel(blue, bg_blue, alpha),
            u8::MAX,
        ];
    }
    Ok(rgba)
}

fn blend_channel(foreground: u8, background: u8, alpha: u16) -> u8 {
    (((foreground as u16 * alpha) + (background as u16 * (255 - alpha))) / 255) as u8
}

fn background_rgb(background: ImageBackground, x: u32, y: u32) -> [u8; 3] {
    match background {
        ImageBackground::Terminal => [0, 0, 0],
        ImageBackground::Black => [0, 0, 0],
        ImageBackground::White => [255, 255, 255],
        ImageBackground::Checker => {
            if ((x / 8) + (y / 8)).is_multiple_of(2) {
                [225, 225, 225]
            } else {
                [150, 150, 150]
            }
        }
    }
}

fn apply_background(image: DynamicImage, background: ImageBackground) -> Result<DynamicImage> {
    if background == ImageBackground::Terminal {
        return Ok(image);
    }
    Ok(DynamicImage::ImageRgba8(rgba_with_background(
        &image, background,
    )?))
}

fn encode_kitty_image(analysis: &ImageAnalysis) -> Result<String> {
    if analysis.format == ImageFormat::Png
        && analysis.orientation == Orientation::NoTransforms
        && analysis.options.background == ImageBackground::Terminal
    {
        let raw = encode_dynamic_png(&rendered_dynamic_image(
            &analysis.decoded,
            analysis.cell_size,
            analysis.options.background,
        )?)?;
        return Ok(encode_kitty_png_image(&raw, analysis.cell_size));
    }

    let image = decode_rgba_image(
        &analysis.decoded,
        analysis.cell_size,
        analysis.options.background,
    )?;
    Ok(encode_kitty_rgba_image(&image, analysis.cell_size))
}

fn encode_kitty_png_image(data: &[u8], cell_size: Option<ImageCellSize>) -> String {
    encode_kitty_payload(&kitty_control_prefix("a=T,f=100,q=2", cell_size), data)
}

fn encode_kitty_rgba_image(image: &DecodedRgbaImage, cell_size: Option<ImageCellSize>) -> String {
    encode_kitty_payload(
        &kitty_control_prefix(
            &format!("a=T,f=32,s={},v={},q=2", image.width, image.height),
            cell_size,
        ),
        &image.pixels,
    )
}

fn kitty_control_prefix(base: &str, cell_size: Option<ImageCellSize>) -> String {
    let mut control = base.to_owned();
    if let Some(size) = cell_size {
        control.push_str(&format!(",c={},r={}", size.columns, size.rows));
    }
    control.push(',');
    control
}

fn encode_kitty_payload(first_control: &str, data: &[u8]) -> String {
    let mut output = String::new();
    let mut chunks = data.chunks(KITTY_CHUNK_BYTES).peekable();
    let mut first = true;

    while let Some(chunk) = chunks.next() {
        let more = usize::from(chunks.peek().is_some());

        output.push_str("\x1b_G");
        if first {
            output.push_str(first_control);
            first = false;
        }
        output.push_str("m=");
        output.push_str(&more.to_string());
        output.push(';');
        BASE64_STANDARD.encode_string(chunk, &mut output);
        output.push_str("\x1b\\");
    }

    output.push('\n');
    output
}

fn encode_sixel_image(analysis: &ImageAnalysis) -> Result<String> {
    let image = decode_rgba_image(
        &analysis.decoded,
        analysis.cell_size,
        analysis.options.background,
    )?;
    encode_sixel_rgba_image(&image)
}

fn encode_sixel_rgba_image(image: &DecodedRgbaImage) -> Result<String> {
    let image = SixelImage::try_from_rgba(image.pixels.clone(), image.width, image.height)
        .context("failed to build sixel image")?;
    let mut output = image.encode().context("failed to encode sixel image")?;
    output.push('\n');
    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{
        DecodedRgbaImage, ImageBackground, ImageCellSize, ImageFit, ImageFormat,
        ImageRenderOptions, debug_image_json, encode_dynamic_png, encode_iterm2_inline_image,
        encode_kitty_png_image, encode_kitty_rgba_image, encode_sixel_rgba_image,
        render_inline_image, resize_to_cell_size, resolve_cell_size, sniff_image_format,
    };
    use image::{DynamicImage, GenericImageView, ImageFormat as EncodedImageFormat, RgbaImage};

    fn one_pixel_png() -> Vec<u8> {
        encode_dynamic_png(&DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            1,
            1,
            image::Rgba([255, 0, 0, 255]),
        )))
        .expect("test png should encode")
    }

    fn one_pixel_bmp() -> Vec<u8> {
        let image =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255])));
        let mut cursor = Cursor::new(Vec::new());
        image
            .write_to(&mut cursor, EncodedImageFormat::Bmp)
            .expect("test bmp should encode");
        cursor.into_inner()
    }

    fn one_pixel_encoded(format: EncodedImageFormat) -> Vec<u8> {
        let image =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255])));
        let mut cursor = Cursor::new(Vec::new());
        image
            .write_to(&mut cursor, format)
            .expect("test image should encode");
        cursor.into_inner()
    }

    fn one_pixel_farbfeld() -> Vec<u8> {
        let mut bytes = Vec::from(&b"farbfeld"[..]);
        bytes.extend_from_slice(&1u32.to_be_bytes());
        bytes.extend_from_slice(&1u32.to_be_bytes());
        bytes.extend_from_slice(&u16::MAX.to_be_bytes());
        bytes.extend_from_slice(&0u16.to_be_bytes());
        bytes.extend_from_slice(&0u16.to_be_bytes());
        bytes.extend_from_slice(&u16::MAX.to_be_bytes());
        bytes
    }

    #[test]
    fn sniffs_common_image_formats() {
        assert_eq!(sniff_image_format(&one_pixel_bmp()), Some(ImageFormat::Bmp));
        assert_eq!(
            sniff_image_format(b"\x89PNG\r\n\x1a\nrest"),
            Some(ImageFormat::Png)
        );
        assert_eq!(
            sniff_image_format(b"\xff\xd8\xff\xe0"),
            Some(ImageFormat::Jpeg)
        );
        assert_eq!(sniff_image_format(b"GIF89a"), Some(ImageFormat::Gif));
        assert_eq!(sniff_image_format(b"II*\0rest"), Some(ImageFormat::Tiff));
        assert_eq!(sniff_image_format(b"MM\0*rest"), Some(ImageFormat::Tiff));
        assert_eq!(sniff_image_format(b"II+\0rest"), Some(ImageFormat::Tiff));
        assert_eq!(sniff_image_format(b"MM\0+rest"), Some(ImageFormat::Tiff));
        assert_eq!(
            sniff_image_format(b"RIFF\x00\x00\x00\x00WEBPrest"),
            Some(ImageFormat::Webp)
        );
        assert_eq!(
            sniff_image_format(b"\0\0\x01\0\x01\0rest"),
            Some(ImageFormat::Ico)
        );
        assert_eq!(sniff_image_format(b"qoifrest"), Some(ImageFormat::Qoi));
        assert_eq!(
            sniff_image_format(b"P6\n1 1\n255\nrest"),
            Some(ImageFormat::Pnm)
        );
        assert_eq!(
            sniff_image_format(b"farbfeldrest"),
            Some(ImageFormat::Farbfeld)
        );
        assert_eq!(
            sniff_image_format(b"#?RADIANCE\nrest"),
            Some(ImageFormat::Hdr)
        );
        assert_eq!(sniff_image_format(b"DDS rest"), Some(ImageFormat::Dds));
        assert_eq!(sniff_image_format(b"BMrest"), None);
        assert_eq!(sniff_image_format(b"\0\0\x01\0\0\0rest"), None);
        assert_eq!(sniff_image_format(b"Println"), None);
        assert_eq!(sniff_image_format(b"hello"), None);
    }

    #[test]
    fn encodes_iterm2_inline_image_osc() {
        let encoded =
            encode_iterm2_inline_image(Some("tiny.png".to_owned()), b"png".to_vec(), None);

        assert!(encoded.starts_with("\x1b]1337;File="));
        assert!(encoded.contains("size=3"));
        assert!(encoded.contains(";inline=1"));
        assert!(encoded.ends_with("\x1b\\\n"));
    }

    #[test]
    fn encodes_kitty_png_image_apc() {
        let encoded = encode_kitty_png_image(b"png", None);

        assert_eq!(encoded, "\x1b_Ga=T,f=100,q=2,m=0;cG5n\x1b\\\n");
    }

    #[test]
    fn encodes_kitty_png_image_in_chunks() {
        let encoded = encode_kitty_png_image(&vec![b'a'; super::KITTY_CHUNK_BYTES + 1], None);

        assert!(encoded.starts_with("\x1b_Ga=T,f=100,q=2,m=1;"));
        assert!(encoded.contains("\x1b\\\x1b_Gm=0;"));
        assert!(encoded.ends_with("\x1b\\\n"));
    }

    #[test]
    fn encodes_kitty_rgba_image_apc() {
        let image = DecodedRgbaImage {
            pixels: vec![255, 0, 0, 255],
            width: 1,
            height: 1,
        };
        let encoded = encode_kitty_rgba_image(&image, None);

        assert!(encoded.starts_with("\x1b_Ga=T,f=32,s=1,v=1,q=2,m=0;"));
        assert!(encoded.ends_with("\x1b\\\n"));
    }

    #[test]
    fn encodes_sixel_rgba_image_dcs() {
        let image = DecodedRgbaImage {
            pixels: vec![255, 0, 0, 255],
            width: 1,
            height: 1,
        };
        let encoded = encode_sixel_rgba_image(&image).expect("sixel should encode");

        assert!(encoded.starts_with("\x1bP"));
        assert!(encoded.ends_with("\x1b\\\n"));
    }

    #[test]
    fn resolves_contained_cell_size_from_terminal_bounds() {
        let size = resolve_cell_size(
            1600,
            900,
            ImageRenderOptions {
                width_cells: None,
                height_cells: None,
                fit: ImageFit::Contain,
                background: ImageBackground::Terminal,
                terminal_columns: Some(100),
                terminal_rows: Some(30),
            },
        )
        .expect("expected automatic cell size");

        assert_eq!(
            size,
            ImageCellSize {
                columns: 86,
                rows: 24
            }
        );
    }

    #[test]
    fn original_fit_does_not_apply_automatic_terminal_bounds() {
        assert_eq!(
            resolve_cell_size(
                1600,
                900,
                ImageRenderOptions {
                    width_cells: None,
                    height_cells: None,
                    fit: ImageFit::Original,
                    background: ImageBackground::Terminal,
                    terminal_columns: Some(100),
                    terminal_rows: Some(30),
                },
            ),
            None
        );
    }

    #[test]
    fn non_tty_image_render_returns_info_fallback() {
        let output = render_inline_image(
            "image.png".as_ref(),
            one_pixel_png(),
            false,
            ImageRenderOptions {
                width_cells: None,
                height_cells: None,
                fit: ImageFit::Contain,
                background: ImageBackground::Terminal,
                terminal_columns: None,
                terminal_rows: None,
            },
        )
        .expect("non-tty image fallback should render");

        assert!(!output.contains_terminal_image);
        assert!(output.output.contains("image.png: PNG 1x1"));
        assert!(output.output.contains("stdout is not a TTY"));
    }

    #[test]
    fn debug_image_json_reports_image_metadata() {
        let json = debug_image_json(
            "image.png".as_ref(),
            &one_pixel_png(),
            false,
            ImageRenderOptions {
                width_cells: Some(10),
                height_cells: None,
                fit: ImageFit::Contain,
                background: ImageBackground::Checker,
                terminal_columns: None,
                terminal_rows: None,
            },
        )
        .expect("debug image metadata should render");

        assert!(json.contains(r#""format": "png""#));
        assert!(json.contains(r#""display_width": 1"#));
        assert!(json.contains(r#""target_cells""#));
        assert!(json.contains(r#""background": "checker""#));
        assert!(json.contains(r#""stdout_is_terminal": false"#));
    }

    #[test]
    fn debug_image_json_reports_bmp_metadata() {
        let json = debug_image_json(
            "image.bmp".as_ref(),
            &one_pixel_bmp(),
            false,
            ImageRenderOptions {
                width_cells: None,
                height_cells: None,
                fit: ImageFit::Contain,
                background: ImageBackground::Terminal,
                terminal_columns: None,
                terminal_rows: None,
            },
        )
        .expect("debug image metadata should render");

        assert!(json.contains(r#""format": "bmp""#));
        assert!(json.contains(r#""display_width": 1"#));
        assert!(json.contains(r#""display_height": 1"#));
    }

    #[test]
    fn debug_image_json_reports_new_low_dependency_format_metadata() {
        for (name, bytes, expected_format) in [
            (
                "image.ico",
                one_pixel_encoded(EncodedImageFormat::Ico),
                "ico",
            ),
            (
                "image.qoi",
                one_pixel_encoded(EncodedImageFormat::Qoi),
                "qoi",
            ),
            (
                "image.pnm",
                one_pixel_encoded(EncodedImageFormat::Pnm),
                "pnm",
            ),
            ("image.ff", one_pixel_farbfeld(), "farbfeld"),
        ] {
            let json = debug_image_json(
                name.as_ref(),
                &bytes,
                false,
                ImageRenderOptions {
                    width_cells: None,
                    height_cells: None,
                    fit: ImageFit::Contain,
                    background: ImageBackground::Terminal,
                    terminal_columns: None,
                    terminal_rows: None,
                },
            )
            .expect("debug image metadata should render");

            assert!(json.contains(&format!(r#""format": "{expected_format}""#)));
            assert!(json.contains(r#""display_width": 1"#));
            assert!(json.contains(r#""display_height": 1"#));
        }
    }

    #[test]
    fn resize_to_cell_size_fits_inside_terminal_cell_pixel_estimate() {
        let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            100,
            100,
            image::Rgba([255, 0, 0, 255]),
        ));

        let resized = resize_to_cell_size(
            &image,
            Some(ImageCellSize {
                columns: 2,
                rows: 3,
            }),
        );

        assert_eq!(resized.dimensions(), (16, 16));
    }

    #[test]
    fn kitty_png_encoding_includes_cell_size() {
        let encoded = encode_kitty_png_image(
            b"png",
            Some(ImageCellSize {
                columns: 12,
                rows: 5,
            }),
        );

        assert!(encoded.starts_with("\x1b_Ga=T,f=100,q=2,c=12,r=5,m=0;"));
    }
}
