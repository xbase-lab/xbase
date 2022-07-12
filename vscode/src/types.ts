
// AUTOGENERATED
// Requests clinets can make
export type Request =
  | { method: "register"; args: RegisterRequest }
  | { method: "build"; args: BuildRequest }
  | { method: "run"; args: RunRequest }
  | { method: "drop"; args: DropRequest }
  | { method: "get_runners" }
  | { method: "get_project_info"; args: GetProjectInfoRequest };

// Server Requests

// Request to build a particular project
export type BuildRequest = { root: string; settings: BuildSettings; operation: Operation };

// Request to Run a particular project.
export type RunRequest = { root: string; settings: BuildSettings; device: DeviceLookup | null; operation: Operation };

// Register a project root
export type RegisterRequest = { root: string };

// Drop a given set of roots to be dropped (i.e. unregistered)
export type DropRequest = { roots: string[] };

// Request to Get `ProjectInfo`
export type GetProjectInfoRequest = { root: string };

// Server Response

// Server Error due to failure while processing a `Request
export type ServerError = { kind: string; msg: string };

// Server Response
export type Response = { data: unknown | null; error: ServerError | null };

// General Transport types

export type ProjectInfo = { watchlist: string[]; targets: { [key: string]: TargetInfo } };

// Target specfic information
export type TargetInfo = { platform: string };

// Represntaiton of Project runners index by Platfrom
export type Runners = { [key: string]: DeviceLookup[] };

// Type of operation for building/ruuning a target/scheme
export enum Operation { Watch = "Watch", Stop = "Stop", Once = "Once" }

// Build Settings used in building/running a target/scheme
export type BuildSettings = { target: string; configuration: string; scheme: string | null };

// Device Lookup information to run built project with
export type DeviceLookup = { name: string; id: string };

// Broadcast server Messages

// Representation of Messages that clients needs to process
export type Message =
  | { type: "Notify"; args: { msg: string; level: MessageLevel } }
  | { type: "Log"; args: { msg: string; level: MessageLevel } }
  | { type: "Execute"; args: Task };

// Message Level
export enum MessageLevel { Trace = "Trace", Debug = "Debug", Info = "Info", Warn = "Warn", Error = "Error", Success = "Success" }

// Tasks that the clients should execute
export type Task =
  | { task: "OpenLogger" }
  | { task: "ReloadLspServer" }
  | { task: "UpdateStatusline"; value: StatuslineState };

// Statusline state
export enum StatuslineState { Clear = "Clear", Failure = "Failure", Processing = "Processing", Running = "Running", Success = "Success", Watching = "Watching" }
