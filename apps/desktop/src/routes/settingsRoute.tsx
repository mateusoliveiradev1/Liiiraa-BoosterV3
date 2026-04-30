import { settingsTrust } from "../../../../packages/ui/src/settingsTrust.js";
import { SettingsTrustSurfaces } from "../components/settings/SettingsTrustSurfaces";
import {
  applyPrivacyConsentToSettings,
  buildPrivacyConsentGateSummary,
  createDefaultPrivacyConsentState
} from "../privacyConsent";

export function SettingsRoute() {
  const consentState = createDefaultPrivacyConsentState();

  return (
    <SettingsTrustSurfaces
      consentGates={buildPrivacyConsentGateSummary(consentState)}
      data={applyPrivacyConsentToSettings(settingsTrust, consentState)}
    />
  );
}
