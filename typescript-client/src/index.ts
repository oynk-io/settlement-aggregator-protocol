import { Address, Contract, Keypair, Networks, rpc, TransactionBuilder, BASE_FEE, xdr } from "@stellar/stellar-sdk";

export type NetworkPassphrase = string;
export type OinkContractIds = { registry: string; payments: string; treasury: string; disputes: string };

export class OinkClient {
  server: rpc.Server;
  ids: OinkContractIds;
  networkPassphrase: string;

  constructor(opts: { rpcUrl: string; ids: OinkContractIds; networkPassphrase?: string }) {
    this.server = new rpc.Server(opts.rpcUrl);
    this.ids = opts.ids;
    this.networkPassphrase = opts.networkPassphrase ?? Networks.TESTNET;
  }

  contract(id: keyof OinkContractIds) { return new Contract(this.ids[id]); }

  async buildTx(source: string, contractName: keyof OinkContractIds, method: string, args: xdr.ScVal[]) {
    const account = await this.server.getAccount(source);
    const contract = this.contract(contractName);
    const tx = new TransactionBuilder(account, { fee: BASE_FEE, networkPassphrase: this.networkPassphrase })
      .addOperation(contract.call(method, ...args))
      .setTimeout(60)
      .build();
    return this.server.prepareTransaction(tx);
  }

  async signAndSend(prepared: any, kp: Keypair) {
    prepared.sign(kp);
    const sent = await this.server.sendTransaction(prepared);
    if (sent.status === "PENDING") return sent;
    throw new Error(`send failed: ${JSON.stringify(sent)}`);
  }
}

export const scAddress = (addr: string) => Address.fromString(addr).toScVal();
export const scU64 = (n: bigint) => xdr.ScVal.scvU64(xdr.Uint64.fromString(n.toString()));
export const scI128 = (n: bigint) => xdr.ScVal.scvI128(new xdr.Int128Parts({ hi: xdr.Int64.fromString("0"), lo: xdr.Uint64.fromString(n.toString()) }));
