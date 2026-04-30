import { settingsTrust } from "../../../../packages/ui/src/settingsTrust.js";
import { SettingsTrustSurfaces } from "../components/settings/SettingsTrustSurfaces";

export function SettingsRoute() {
  return <SettingsTrustSurfaces data={settingsTrust} />;
}
