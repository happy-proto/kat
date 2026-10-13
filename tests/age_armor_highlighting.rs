use std::{path::Path, process::Command};

const ARMORED: &str = include_str!("../testdata/fixtures/armor/message.age");

#[test]
fn age_armor_detects_by_content_and_reuses_pem() {
    for source in [
        ARMORED.to_owned(),
        ARMORED.replace('\n', "\r\n"),
        ARMORED.trim_end().to_owned(),
    ] {
        for path in [
            Some(Path::new("message.age")),
            Some(Path::new("message")),
            None,
        ] {
            assert_eq!(kat::detected_language_name(path, &source), Some("pem"));
        }
    }
}

#[test]
fn age_extension_alone_and_mismatched_boundaries_do_not_select_pem() {
    for source in [
        "plain text\n".to_owned(),
        "age-encryption.org/v1\n-> X25519 example\n--- example\n".to_owned(),
        ARMORED.replace(
            "-----END AGE ENCRYPTED FILE-----",
            "-----END CERTIFICATE-----",
        ),
        ARMORED.replace("AGE ENCRYPTED FILE", "age encrypted file"),
        ARMORED.replace("-----END AGE ENCRYPTED FILE-----", ""),
    ] {
        assert_ne!(
            kat::detected_language_name(Some(Path::new("message.age")), &source),
            Some("pem")
        );
    }
}

#[test]
fn cli_highlights_age_boundaries_label_and_padded_base64() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/fixtures/armor/message.age");
    let output = Command::new(env!("CARGO_BIN_EXE_kat"))
        .args(["--hyperlinks", "never"])
        .arg(&path)
        .env("COLORTERM", "truecolor")
        .env_remove("NO_COLOR")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered = String::from_utf8(output.stdout).unwrap();
    for token in ["-----BEGIN ", "-----END "] {
        assert!(
            rendered.contains(&format!("\x1b[3m\x1b[38;2;80;250;123m{token}")),
            "{rendered:?}"
        );
    }
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mAGE ENCRYPTED FILE"),
        "{rendered:?}"
    );
    for body in ARMORED.lines().skip(1).take(5) {
        assert!(
            rendered.contains(&format!("\x1b[38;2;189;147;249m{body}")),
            "{rendered:?}"
        );
    }

    let plain = Command::new(env!("CARGO_BIN_EXE_kat"))
        .args(["--hyperlinks", "never"])
        .arg(path)
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    assert!(plain.status.success());
    assert_eq!(String::from_utf8(plain.stdout).unwrap(), ARMORED);
}
