export interface MultiplayerHost {
  setState(state: unknown): void
  start(sessionId: string): Promise<void>
  dispose(): Promise<void>
  closed: Promise<void>
}

export function createTogetherMultiplayer(options: {
  localNetwork: unknown
  sharedFiles: unknown
  peerConnectionProvider: unknown
  waitForIceServersBeforePeer: boolean
  logger: { emit: (level: string, event: string, data: unknown) => void }
  createTelemetryAttempt: () => unknown
  setDownloadPort: (port: number) => Promise<void>
  roomApi: unknown
}): MultiplayerHost {
  return {
    setState() {},
    async start() {},
    async dispose() {},
    closed: Promise.resolve(),
  } as MultiplayerHost
}
