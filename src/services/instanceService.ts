/**
 * instanceService — facade
 * 实际实现拆分至 ./instanceService/ 子模块，此处统一 re-export 以保持导入路径不变。
 */
export * from './instanceService/types';
export * from './instanceService/crud';
export * from './instanceService/lifecycle';
export * from './instanceService/autoSwitcher';
export * from './instanceService/projects';
export * from './instanceService/scoring';
export * from './instanceService/sync';
