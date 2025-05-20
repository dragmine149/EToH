const { Network } = require('../newJs/network');

describe('Network', () => {
  let network;

  beforeEach(() => {
    network = new Network();
  });

  test('should retry until result is successful', async () => {
    global.fetch = jest.fn()
      .mockRejectedValueOnce(new Error('Network Error'))
      .mockResolvedValueOnce({ ok: true });

    const response = await network.retryTilResult(new Request('https://example.com'));
    expect(response.ok).toBe(true);
  });
});
