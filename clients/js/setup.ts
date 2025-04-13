process.env['RPC_PORT'] = process.env['RPC_PORT'] || '8899';
const RPC_PORT = process.env['RPC_PORT'];

module.exports = async function() {
  // Wait for RPC to be ready
  await waitForRpc();
}

async function waitForRpc(): Promise<boolean> {
  while (true) {
    if (await (validateRpc())) {
      console.log("RPC validation success");
      return true;
    }
  }
}

async function validateRpc(): Promise<boolean> {
  const data = await fetch(`http://127.0.0.1:${RPC_PORT}`, {
    method: 'post',
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: "1",
      method: "getHealth"
    }),
    headers: { 'Content-Type': 'application/json' },
  })
  .then((res) => res.json())
  .catch(() => null)

  return data?.result === 'ok';
}