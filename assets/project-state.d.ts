/**
 * TypeScript type definitions for project-state.json
 */

export interface ProjectState {
  project: {
    name: string;
    description: string;
    tagline: string;
    version: string;
  };
  package: {
    name: string;
    binary: {
      main: string;
      allowed_additional: string[];
    };
  };
  organization: {
    name: string;
    shortName: string;
    website: string;
    email: string;
  };
  repository: {
    owner: string;
    name: string;
    url: string;
    issues: string;
    discussions: string;
  };
  deprecated: {
    binaryNames: string[];
    repositoryUrls: string[];
  };
  badges: {
    ci?: {
      label: string;
      workflow: string;
    };
    coverage?: {
      label: string;
      percentage: number;
      color: string;
    };
    license: {
      type: string;
      url: string;
    };
  };
  installation: {
    paths: {
      global: string;
      local: string;
    };
    installer: {
      script: string;
      url: string;
    };
  };
  mcp: {
    serverName: string;
    protocol: string;
    configPath: {
      macOS: string;
      linux: string;
      windows: string;
    };
  };
}