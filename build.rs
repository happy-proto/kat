use shadow_rs::{BuildPattern, ShadowBuilder};

fn main() {
    ShadowBuilder::builder()
        .build_pattern(BuildPattern::RealTime)
        .build()
        .expect("failed to generate shadow build metadata");
}
