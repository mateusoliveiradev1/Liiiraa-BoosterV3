import { desktopCommandCenterState } from "../adapters/desktopState";
import { SettingsTrustSurfaces } from "../components/settings/SettingsTrustSurfaces";

export function SettingsRoute() {
  return (
    <SettingsTrustSurfaces
      consentGates={desktopCommandCenterState.routes.settingsConsentGates}
      data={desktopCommandCenterState.routes.settings}
    />
  );
}
