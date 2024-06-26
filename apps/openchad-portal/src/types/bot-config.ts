/* eslint-disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export interface BotConfig {
  categorizePrompt: string[];
  endpoints: {
    [k: string]: ConfigEndpoint;
  };
  fallbackEndpoint: string;
  helpPrompt: string[];
  macros: {
    [k: string]: {
      [k: string]: {
        [k: string]: string;
      };
    };
  };
  messageHistory: number;
  props: {
    [k: string]: string;
  };
  providers: {
    [k: string]: ConfigProvider;
  };
  responses: {
    [k: string]: ConfigResponse;
  };
  [k: string]: unknown;
}
export interface ConfigEndpoint {
  categorization: string;
  designation: string;
  icon: string;
  id: string;
  task: string;
  [k: string]: unknown;
}
export interface ConfigProvider {
  props: {
    [k: string]: string;
  };
  provider: string;
  transform: {
    [k: string]: string;
  };
  [k: string]: unknown;
}
export interface ConfigResponse {
  footer?: string | null;
  prompt: string[];
  transform?: {
    [k: string]: string;
  } | null;
  [k: string]: unknown;
}
