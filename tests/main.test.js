const { tryCatch, noSyncTryCatch } = require('../newJs/main');

describe('tryCatch', () => {
  test('should return data when promise resolves', async () => {
    const result = await tryCatch(Promise.resolve('Success'));

    expect(result.data).toBe('Success');
    expect(result.error).toBeNull();
  });

  test('should return error when promise rejects', async () => {
    const result = await tryCatch(Promise.reject(new Error('Failure')));

    expect(result.data).toBeNull();
    expect(result.error.message).toBe('Failure');
  });
});
