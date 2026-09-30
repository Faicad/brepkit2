/**
 * @faicad/brepkit2-wasm — TypeScript bindings for the brepkit2 CAD kernel.
 *
 * @example
 * ```ts
 * import { initBrepkit } from '@faicad/brepkit2-wasm';
 *
 * await initBrepkit();
 * // Use brepkit functions...
 * ```
 */

export { initBrepkit, isInitialized } from './init.js';
export type { Point3, Vec3, SolidHandle, FaceHandle, EdgeHandle } from './types.js';
