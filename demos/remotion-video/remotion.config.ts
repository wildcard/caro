import { Config } from "@remotion/cli/config";

Config.setVideoImageFormat("jpeg");
Config.setOverwriteOutput(true);
// Default concurrency = number of CPU cores. Setting it to 1 was a bad
// idea for a 900-frame render — leave it unset.
