export interface TogetherRoomApi {
  getBaseUrl(): string
}

export function createTogetherRoomApi(options: {
  getBaseUrl: () => string
  fetch: (url: string, init?: unknown) => Promise<unknown>
}): TogetherRoomApi {
  return {
    getBaseUrl: options.getBaseUrl,
  }
}
