pub struct GithubHeaders;

impl GithubHeaders {
    pub const EVENT: &'static str = "x-github-event";
    // pub const DELIVERY: &'static str = "x-github-delivery";
    pub const SIGNATURE_256: &'static str = "x-hub-signature-256";
    // pub const HOOK_ID: &'static str = "x-github-hook-id";
    // pub const HOOK_INSTALLATION_TARGET_ID: &'static str = "x-github-hook-installation-target-id";
    // pub const HOOK_INSTALLATION_TARGET_TYPE: &'static str =
    //     "x-github-hook-installation-target-type";
}