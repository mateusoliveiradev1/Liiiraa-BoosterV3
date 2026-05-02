use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;
use url::Url;

const DEFAULT_RELEASE_CHANNEL: ReleaseChannel = ReleaseChannel::Stable;
const UPDATE_ENDPOINT_ORIGIN: &str = "https://updates.liiiraa.example";
const WINDOWS_INSTALL_MODE: &str = "passive";

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ReleaseChannel {
    Dev,
    Beta,
    Stable,
}

impl ReleaseChannel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Beta => "beta",
            Self::Stable => "stable",
        }
    }

    fn rollout_policy(self) -> &'static str {
        match self {
            Self::Dev => "internal testing, can break",
            Self::Beta => "signed soak for updater, privileged-agent, and Lab changes",
            Self::Stable => "signed public release after beta soak or approval",
        }
    }

    fn all() -> [Self; 3] {
        [Self::Dev, Self::Beta, Self::Stable]
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CheckSignedUpdateRequest {
    channel: Option<ReleaseChannel>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdaterConfigurationResponse {
    default_channel: ReleaseChannel,
    channels: Vec<UpdaterChannelResponse>,
    create_updater_artifacts: bool,
    dangerous_insecure_transport_protocol: bool,
    private_key_embedded: bool,
    signature_required: bool,
    windows_install_mode: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdaterChannelResponse {
    id: ReleaseChannel,
    endpoint: String,
    rollout_policy: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SignedUpdateCheckResponse {
    channel: ReleaseChannel,
    current_version: Option<String>,
    endpoint: String,
    signature_required: bool,
    update_available: bool,
    version: Option<String>,
    windows_install_mode: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdaterErrorResponse {
    reason: String,
    message: String,
}

pub(crate) fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    app.handle()
        .plugin(tauri_plugin_updater::Builder::new().build())?;
    Ok(())
}

#[tauri::command]
pub(crate) fn get_updater_configuration(
) -> Result<UpdaterConfigurationResponse, UpdaterErrorResponse> {
    updater_configuration()
}

#[tauri::command]
pub(crate) async fn check_signed_update(
    app: AppHandle,
    payload: CheckSignedUpdateRequest,
) -> Result<SignedUpdateCheckResponse, UpdaterErrorResponse> {
    let channel = payload.channel.unwrap_or(DEFAULT_RELEASE_CHANNEL);
    let endpoint_template = updater_endpoint_template(channel);
    let endpoint = updater_endpoint(channel)?;

    let update = app
        .updater_builder()
        .endpoints(vec![endpoint.clone()])
        .map_err(UpdaterErrorResponse::from)?
        .build()
        .map_err(UpdaterErrorResponse::from)?
        .check()
        .await
        .map_err(UpdaterErrorResponse::from)?;

    Ok(SignedUpdateCheckResponse {
        channel,
        current_version: update.as_ref().map(|metadata| metadata.current_version.clone()),
        endpoint: endpoint_template,
        signature_required: true,
        update_available: update.is_some(),
        version: update.as_ref().map(|metadata| metadata.version.clone()),
        windows_install_mode: WINDOWS_INSTALL_MODE,
    })
}

fn updater_configuration() -> Result<UpdaterConfigurationResponse, UpdaterErrorResponse> {
    Ok(UpdaterConfigurationResponse {
        default_channel: DEFAULT_RELEASE_CHANNEL,
        channels: ReleaseChannel::all()
            .into_iter()
            .map(updater_channel_response)
            .collect::<Result<Vec<_>, _>>()?,
        create_updater_artifacts: true,
        dangerous_insecure_transport_protocol: false,
        private_key_embedded: false,
        signature_required: true,
        windows_install_mode: WINDOWS_INSTALL_MODE,
    })
}

fn updater_channel_response(
    channel: ReleaseChannel,
) -> Result<UpdaterChannelResponse, UpdaterErrorResponse> {
    let endpoint = updater_endpoint_template(channel);
    Url::parse(&endpoint).map_err(UpdaterErrorResponse::from)?;

    Ok(UpdaterChannelResponse {
        id: channel,
        endpoint,
        rollout_policy: channel.rollout_policy(),
    })
}

fn updater_endpoint(channel: ReleaseChannel) -> Result<Url, UpdaterErrorResponse> {
    Url::parse(&updater_endpoint_template(channel)).map_err(UpdaterErrorResponse::from)
}

fn updater_endpoint_template(channel: ReleaseChannel) -> String {
    format!(
        "{UPDATE_ENDPOINT_ORIGIN}/{}/{{{{target}}}}/{{{{arch}}}}/{{{{current_version}}}}",
        channel.as_str()
    )
}

impl From<tauri_plugin_updater::Error> for UpdaterErrorResponse {
    fn from(error: tauri_plugin_updater::Error) -> Self {
        Self {
            reason: "updater_error".to_owned(),
            message: error.to_string(),
        }
    }
}

impl From<url::ParseError> for UpdaterErrorResponse {
    fn from(error: url::ParseError) -> Self {
        Self {
            reason: "invalid_update_endpoint".to_owned(),
            message: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_is_the_default_release_channel() {
        let config = updater_configuration().expect("updater config should be valid");

        assert_eq!(config.default_channel, ReleaseChannel::Stable);
        assert!(config.signature_required);
        assert!(config.create_updater_artifacts);
        assert!(!config.dangerous_insecure_transport_protocol);
        assert!(!config.private_key_embedded);
    }

    #[test]
    fn channel_endpoints_are_https_and_scoped() {
        for channel in ReleaseChannel::all() {
            let endpoint_template = updater_endpoint_template(channel);
            let endpoint = updater_endpoint(channel).expect("channel endpoint should parse");

            assert_eq!(endpoint.scheme(), "https");
            assert!(endpoint_template.contains(channel.as_str()));
            assert!(endpoint_template.contains("{{target}}"));
            assert!(endpoint_template.contains("{{arch}}"));
            assert!(endpoint_template.contains("{{current_version}}"));
        }
    }
}
