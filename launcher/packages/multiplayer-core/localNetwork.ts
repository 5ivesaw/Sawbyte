export interface LocalSocket {
  readonly id: string
  write(data: ArrayBuffer): void
  end(): void
  pause(): void
  resume(): void
  close(): void
  onData(listener: (data: ArrayBuffer) => void): void
  onWrite(listener: (length: number) => void): void
  onDrain(listener: () => void): void
  onClose(listener: () => void): void
  onError(listener: (error: Error) => void): void
}

export interface LocalServer {
  readonly id: string
  accept(socket: LocalSocket): void
  close(): void
  onConnection(listener: (socket: LocalSocket) => void): void
}

export interface LocalLanServer {
  readonly id: string
  readonly port: number
  close(): void
}

export interface SharedFiles {
  share(instancePath: string | undefined, files: string[]): Promise<void>
  open(rawPath: string): Promise<TransferReadable>
}

export interface LocalNetwork {
  connect(port: number): Promise<LocalSocket>
  listen(requestedPort: number, localOnly?: boolean): Promise<number>
  discoverLan(listener: (server: LocalLanServer) => void): Promise<void>
  stopLanDiscovery(listener: (server: LocalLanServer) => void): void
  broadcastLan(server: LocalLanServer): Promise<void>
  dispose(): Promise<void>
}
