export interface TransferReadable {
  pause(): void
  resume(): void
  close(): void
  onData(listener: (data: ArrayBuffer) => void): void
  onClose(listener: () => void): void
  onError(listener: (error: Error) => void): void
}
