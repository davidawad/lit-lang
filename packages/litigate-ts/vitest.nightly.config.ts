import { defineConfig, mergeConfig } from "vitest/config";
import config from "./vitest.config.js";

const fullCorpusPattern = /(?:check matches the CLI > (?:everything|everything, strict)|finds a matter's packages in the embedded library)/;

export default mergeConfig(
  config,
  defineConfig({
    test: {
      testNamePattern: fullCorpusPattern,
    },
  }),
);
