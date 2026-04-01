use regex::Regex;

pub fn build_preview_query() {
    sql!(
        r#"
        WITH ranked AS (
            SELECT id, name, enabled
            FROM themes
            WHERE enabled = true
        )
        SELECT *
        FROM ranked
        ORDER BY name DESC
        "#
    );
    let _ = client.query("SELECT id FROM themes WHERE slug = $1", &[]);
    let _ = client.prepare_typed("SELECT id FROM themes WHERE slug = $1", &[]);
    let _ = client.simple_query("SELECT id FROM themes");

    let slug_pattern = regex!(r"^(?P<kind>theme)-(?P<slug>[a-z0-9_-]+)$");
    let route_pattern = Regex::new(r"(?i)^(?P<section>theme|preview):(?P<value>\w+)$").unwrap();
    let unicode_pattern =
        Regex::new(r"^(?P<section>theme|preview):(?P<value>\p{L}+)$").unwrap();
    let escaped_pattern =
        Regex::new("^(?P<escaped_section>theme|preview):(?P<escaped_value>\\w+)$").unwrap();
    let repeated_word = regex!(r"(?P<word>[[:alpha:]]+)-(?P=word)");
    let verbose_pattern = regex::RegexBuilder::new(
        r"(?x)
        ^(?P<kind>theme)
        :
        (?P<slug>[a-z0-9_-]+)$"
    )
    .build()
    .unwrap();

    println!("{slug_pattern:?} {route_pattern:?} {unicode_pattern:?} {escaped_pattern:?} {repeated_word:?} {verbose_pattern:?}");
}
