import { registerCommonViewerSuite, registerDioxusThemeSuite, registerGraph3dProfilingSuite } from './shared/suites';
import { SPEC_VIEWER } from './shared/viewers';

registerCommonViewerSuite(SPEC_VIEWER);
registerDioxusThemeSuite(SPEC_VIEWER);
registerGraph3dProfilingSuite(SPEC_VIEWER);