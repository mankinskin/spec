import { registerClientLogApiSuite, registerTracingConsoleSuite } from './shared/suites';
import { SPEC_VIEWER } from './shared/viewers';

registerTracingConsoleSuite(SPEC_VIEWER);
registerClientLogApiSuite(SPEC_VIEWER);