import React, { useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import {
  Activity,
  AlertTriangle,
  ArrowDownLeft,
  ArrowRight,
  ArrowUpRight,
  Banknote,
  Bell,
  CheckCircle2,
  ChevronDown,
  CircleDollarSign,
  Clock3,
  Copy,
  FileCheck2,
  FileText,
  Filter,
  Home,
  Landmark,
  Lock,
  Menu,
  MessageSquare,
  RefreshCw,
  Search,
  Settings,
  ShieldCheck,
  UserRound,
  Wallet,
  X,
  XCircle,
  Plus,
  Send,
  UsersRound,
  Check,
  Hash,
  ReceiptText,
  Coins,
  Building2,
  Info,
  ExternalLink,
} from "lucide-react";
import "./index.css";

const PAYMENT_KIND = {
  FIAT_TO_CRYPTO: "FiatToCrypto",
  CRYPTO_TO_FIAT: "CryptoToFiat",
  FIAT_TO_FIAT: "FiatToFiat",
};

const FUNDING_STATUS = {
  PENDING_QUOTE: "PendingQuote",
  QUOTE_SET: "QuoteSet",
  FIAT_CONFIRMED: "FiatConfirmed",
  READY: "Ready",
};

const PAYMENT_STATUS = {
  CREATED: "Created",
  SOURCE_ASSIGNED: "SourceSettlerAssigned",
  DEST_ASSIGNED: "DestinationSettlerAssigned",
  SETTLEMENT_IN_PROGRESS: "SettlementInProgress",
  READY_FOR_CLAIM: "ReadyForClaim",
  COMPLETED: "Completed",
  CANCELLED: "Cancelled",
  REFUNDED: "Refunded",
  DISPUTED: "Disputed",
};

const CURRENCIES = [
  { code: 840, symbol: "USD", name: "US Dollar" },
  { code: 414, symbol: "KWD", name: "Kuwaiti Dinar" },
  { code: 566, symbol: "NGN", name: "Nigerian Naira" },
  { code: 826, symbol: "GBP", name: "British Pound" },
  { code: 978, symbol: "EUR", name: "Euro" },
  { code: 124, symbol: "CAD", name: "Canadian Dollar" },
  { code: 404, symbol: "KES", name: "Kenyan Shilling" },
  { code: 936, symbol: "GHS", name: "Ghanaian Cedi" },
  { code: 710, symbol: "ZAR", name: "South African Rand" },
];

const USERS = [
  {
    id: "usr_001",
    name: "Aisha M.",
    email: "aisha@example.com",
    wallet: "GDK4...92HF",
    country: "Kuwait",
    kyc: "Verified",
  },
  {
    id: "usr_002",
    name: "Mohammed A.",
    email: "mohammed@example.com",
    wallet: "GCB7...K29P",
    country: "Kuwait",
    kyc: "Verified",
  },
  {
    id: "usr_003",
    name: "Chinedu O.",
    email: "chinedu@example.com",
    wallet: "GAF9...88QT",
    country: "Nigeria",
    kyc: "Verified",
  },
  {
    id: "usr_004",
    name: "Sara K.",
    email: "sara@example.com",
    wallet: "GDA2...88LM",
    country: "Kuwait",
    kyc: "Verified",
  },
  {
    id: "usr_005",
    name: "Kwame B.",
    email: "kwame@example.com",
    wallet: "GBB1...98GH",
    country: "Ghana",
    kyc: "Verified",
  },
  {
    id: "usr_006",
    name: "Grace N.",
    email: "grace@example.com",
    wallet: "GCR8...53KE",
    country: "Kenya",
    kyc: "Verified",
  },
  {
    id: "usr_007",
    name: "Yusuf A.",
    email: "yusuf@example.com",
    wallet: "GDX7...02NG",
    country: "Nigeria",
    kyc: "Review",
  },
  {
    id: "usr_008",
    name: "Noura S.",
    email: "noura@example.com",
    wallet: "GFS1...KWD",
    country: "Kuwait",
    kyc: "Verified",
  },
];

const SETTLERS = [
  {
    id: "stl_source_kwd",
    name: "Kuwait USDC Desk",
    address: "GDUS...KWD1",
    type: "source",
    liquidity: "84,000 USDC",
    region: "Kuwait",
  },
  {
    id: "stl_source_ngn",
    name: "Lagos Liquidity Desk",
    address: "GDLG...NGN1",
    type: "source",
    liquidity: "120,000 USDC",
    region: "Nigeria",
  },
  {
    id: "stl_dest_kwd",
    name: "Kuwait Bank Payout Desk",
    address: "GDPY...KWD1",
    type: "destination",
    liquidity: "32,000 KWD",
    region: "Kuwait",
  },
  {
    id: "stl_dest_ngn",
    name: "Lagos Payout Desk",
    address: "GDPO...NGN1",
    type: "destination",
    liquidity: "190,000,000 NGN",
    region: "Nigeria",
  },
  {
    id: "stl_dest_ghs",
    name: "Accra Payout Desk",
    address: "GDAC...GHS1",
    type: "destination",
    liquidity: "400,000 GHS",
    region: "Ghana",
  },
];

function currency(code) {
  if (code == null) return "USDC";
  return (
    CURRENCIES.find((c) => c.code === Number(code))?.symbol || String(code)
  );
}

function money(value, code = "USDC") {
  const symbol = typeof code === "number" ? currency(code) : code;
  return `${Number(value || 0).toLocaleString(undefined, {
    maximumFractionDigits: 2,
  })} ${symbol}`;
}

function shortKind(kind) {
  return (
    {
      FiatToCrypto: "Fiat → Crypto",
      CryptoToFiat: "Crypto → Fiat",
      FiatToFiat: "Fiat → Fiat",
    }[kind] || kind
  );
}

function statusLabel(s) {
  return String(s || "")
    .replace(/([A-Z])/g, " $1")
    .trim();
}

function nowHash(prefix = "0x") {
  return (
    prefix +
    Math.random().toString(16).slice(2, 10) +
    Math.random().toString(16).slice(2, 10)
  );
}

function ref32(prefix) {
  return `${prefix}_${Math.random()
    .toString(16)
    .slice(2, 10)}_${Date.now().toString(36)}`;
}

function makePayment({
  id,
  creatorId,
  kind,
  origin,
  destination,
  destinationAmount,
  sourceAmount = null,
  escrowAmount = null,
  fundingStatus = FUNDING_STATUS.PENDING_QUOTE,
  status = PAYMENT_STATUS.CREATED,
  sourceSettler = null,
  destinationSettler = null,
  fiatEvidenceHash = null,
  quoteEvidenceHash = null,
  settlementEvidenceHash = null,
}) {
  const user = USERS.find((u) => u.id === creatorId) || USERS[0];
  return {
    id,
    creator: user,
    senderRef: ref32("sender"),
    recipientRef: ref32("recipient"),
    route: { origin, destination },
    paymentKind: kind,
    sourceAmount,
    destinationAmount,
    escrowAmount,
    fundingStatus,
    status,
    settlers: {
      source: sourceSettler,
      destination: destinationSettler,
    },
    quoteEvidenceHash,
    fiatEvidenceHash,
    settlementEvidenceHash,
    createdAt: "2026-06-29 10:30",
    deadline: "2026-06-30 10:30",
    risk: kind === PAYMENT_KIND.CRYPTO_TO_FIAT ? "Medium" : "Low",
    audit: ["Payment created"],
  };
}

const initialPayments = [
  makePayment({
    id: 1,
    creatorId: "usr_001",
    kind: PAYMENT_KIND.FIAT_TO_CRYPTO,
    origin: 414,
    destination: null,
    destinationAmount: 1000,
  }),
  makePayment({
    id: 2,
    creatorId: "usr_002",
    kind: PAYMENT_KIND.CRYPTO_TO_FIAT,
    origin: null,
    destination: 414,
    destinationAmount: 920,
    sourceAmount: 3000,
    escrowAmount: 3000,
    fundingStatus: FUNDING_STATUS.QUOTE_SET,
    quoteEvidenceHash: nowHash(),
  }),
  makePayment({
    id: 3,
    creatorId: "usr_003",
    kind: PAYMENT_KIND.FIAT_TO_FIAT,
    origin: 566,
    destination: 840,
    destinationAmount: 1200,
    sourceAmount: 1850000,
    escrowAmount: 1200,
    fundingStatus: FUNDING_STATUS.FIAT_CONFIRMED,
    fiatEvidenceHash: nowHash(),
    quoteEvidenceHash: nowHash(),
  }),
  makePayment({
    id: 4,
    creatorId: "usr_004",
    kind: PAYMENT_KIND.FIAT_TO_CRYPTO,
    origin: 414,
    destination: null,
    destinationAmount: 650,
    sourceAmount: 200,
    escrowAmount: 650,
    fundingStatus: FUNDING_STATUS.FIAT_CONFIRMED,
    quoteEvidenceHash: nowHash(),
    fiatEvidenceHash: nowHash(),
  }),
  makePayment({
    id: 5,
    creatorId: "usr_005",
    kind: PAYMENT_KIND.CRYPTO_TO_FIAT,
    origin: null,
    destination: 936,
    destinationAmount: 7200,
    sourceAmount: 600,
    escrowAmount: 600,
    fundingStatus: FUNDING_STATUS.READY,
    quoteEvidenceHash: nowHash(),
    status: PAYMENT_STATUS.CREATED,
  }),
  makePayment({
    id: 6,
    creatorId: "usr_006",
    kind: PAYMENT_KIND.FIAT_TO_FIAT,
    origin: 404,
    destination: 414,
    destinationAmount: 90,
    sourceAmount: 38000,
    escrowAmount: 295,
    fundingStatus: FUNDING_STATUS.READY,
    quoteEvidenceHash: nowHash(),
    fiatEvidenceHash: nowHash(),
    status: PAYMENT_STATUS.CREATED,
    sourceSettler: {
      settler: SETTLERS[1],
      amountGives: 295,
      amountReceives: 38000,
      accepted: false,
      confirmed: true,
      proofHash: null,
    },
  }),
  makePayment({
    id: 7,
    creatorId: "usr_007",
    kind: PAYMENT_KIND.FIAT_TO_FIAT,
    origin: 566,
    destination: 414,
    destinationAmount: 250,
    sourceAmount: 386000,
    escrowAmount: 815,
    fundingStatus: FUNDING_STATUS.READY,
    quoteEvidenceHash: nowHash(),
    fiatEvidenceHash: nowHash(),
    status: PAYMENT_STATUS.DEST_ASSIGNED,
    sourceSettler: {
      settler: SETTLERS[1],
      amountGives: 815,
      amountReceives: 386000,
      accepted: false,
      confirmed: true,
      proofHash: null,
    },
    destinationSettler: {
      settler: SETTLERS[2],
      amountGives: 250,
      amountReceives: 815,
      accepted: false,
      confirmed: false,
      proofHash: null,
    },
  }),
  makePayment({
    id: 8,
    creatorId: "usr_008",
    kind: PAYMENT_KIND.CRYPTO_TO_FIAT,
    origin: null,
    destination: 566,
    destinationAmount: 1500000,
    sourceAmount: 1000,
    escrowAmount: 1000,
    fundingStatus: FUNDING_STATUS.READY,
    quoteEvidenceHash: nowHash(),
    status: PAYMENT_STATUS.SETTLEMENT_IN_PROGRESS,
    destinationSettler: {
      settler: SETTLERS[3],
      amountGives: 1500000,
      amountReceives: 1000,
      accepted: true,
      confirmed: false,
      proofHash: null,
    },
  }),
];

function cx(...classes) {
  return classes.filter(Boolean).join(" ");
}

function Badge({ children, tone = "slate" }) {
  const styles = {
    slate: "bg-slate-100 text-slate-700 ring-slate-200",
    blue: "bg-blue-50 text-blue-700 ring-blue-200",
    violet: "bg-violet-50 text-violet-700 ring-violet-200",
    green: "bg-emerald-50 text-emerald-700 ring-emerald-200",
    amber: "bg-amber-50 text-amber-700 ring-amber-200",
    red: "bg-rose-50 text-rose-700 ring-rose-200",
  };
  return (
    <span
      className={cx(
        "inline-flex items-center rounded-full px-2.5 py-1 text-xs font-bold ring-1",
        styles[tone]
      )}
    >
      {children}
    </span>
  );
}

function paymentTone(status) {
  if (status === PAYMENT_STATUS.COMPLETED) return "green";
  if (
    status === PAYMENT_STATUS.DISPUTED ||
    status === PAYMENT_STATUS.CANCELLED ||
    status === PAYMENT_STATUS.REFUNDED
  )
    return "red";
  if (status === PAYMENT_STATUS.READY_FOR_CLAIM) return "blue";
  if (
    status === PAYMENT_STATUS.SETTLEMENT_IN_PROGRESS ||
    status === PAYMENT_STATUS.DEST_ASSIGNED ||
    status === PAYMENT_STATUS.SOURCE_ASSIGNED
  )
    return "amber";
  return "slate";
}

function fundingTone(status) {
  if (status === FUNDING_STATUS.READY) return "green";
  if (status === FUNDING_STATUS.FIAT_CONFIRMED) return "blue";
  if (status === FUNDING_STATUS.QUOTE_SET) return "amber";
  return "slate";
}

function App() {
  const [view, setView] = useState("user");
  const [userId, setUserId] = useState("usr_001");
  const [payments, setPayments] = useState(initialPayments);
  const [selectedId, setSelectedId] = useState(1);
  const [modal, setModal] = useState(null);
  const [toast, setToast] = useState("");

  const selected = payments.find((p) => p.id === selectedId) || payments[0];
  const currentUser = USERS.find((u) => u.id === userId) || USERS[0];

  function patchPayment(id, updater, auditText) {
    setPayments((prev) =>
      prev.map((p) => {
        if (p.id !== id) return p;
        const next =
          typeof updater === "function" ? updater(p) : { ...p, ...updater };
        return {
          ...next,
          audit: auditText ? [...next.audit, auditText] : next.audit,
        };
      })
    );
    setToast(auditText || "Payment updated");
    setTimeout(() => setToast(""), 1800);
  }

  function createPayment(form) {
    const nextId = Math.max(...payments.map((p) => p.id)) + 1;
    const kind = form.paymentKind;
    const origin =
      kind === PAYMENT_KIND.CRYPTO_TO_FIAT ? null : Number(form.origin);
    const destination =
      kind === PAYMENT_KIND.FIAT_TO_CRYPTO ? null : Number(form.destination);
    const p = makePayment({
      id: nextId,
      creatorId: userId,
      kind,
      origin,
      destination,
      destinationAmount: Number(form.destinationAmount),
    });
    setPayments((prev) => [p, ...prev]);
    setSelectedId(nextId);
    setModal(null);
    setToast(`Payment #${nextId} created`);
  }

  return (
    <div className="min-h-screen bg-slate-50">
      <Topbar
        view={view}
        setView={setView}
        currentUser={currentUser}
        setUserId={setUserId}
        onCreate={() => setModal({ type: "create" })}
      />
      <main className="mx-auto grid max-w-[1500px] gap-6 px-4 py-6 lg:grid-cols-[360px_1fr]">
        <aside className="space-y-4">
          <Summary payments={payments} />
          <PaymentList
            payments={payments}
            selectedId={selectedId}
            setSelectedId={setSelectedId}
            view={view}
            currentUser={currentUser}
          />
        </aside>
        <section className="min-w-0">
          {view === "user" ? (
            <UserPaymentView
              payment={selected}
              currentUser={currentUser}
              patchPayment={patchPayment}
            />
          ) : view === "admin" ? (
            <AdminPaymentView
              payment={selected}
              setModal={setModal}
              patchPayment={patchPayment}
            />
          ) : (
            <SettlerView
              payment={selected}
              setModal={setModal}
              patchPayment={patchPayment}
            />
          )}
        </section>
      </main>

      {modal?.type === "create" && (
        <CreatePaymentModal
          onClose={() => setModal(null)}
          onSubmit={createPayment}
        />
      )}
      {modal?.type === "quote" && (
        <QuoteModal
          payment={selected}
          onClose={() => setModal(null)}
          onSubmit={(values) => {
            patchPayment(
              selected.id,
              (p) => ({
                ...p,
                sourceAmount: Number(values.sourceAmount),
                escrowAmount: Number(values.escrowAmount),
                quoteEvidenceHash: values.evidenceHash || nowHash(),
                fundingStatus: FUNDING_STATUS.QUOTE_SET,
              }),
              `Quote set: source ${values.sourceAmount}, escrow ${values.escrowAmount} USDC`
            );
            setModal(null);
          }}
        />
      )}
      {modal?.type === "fiat" && (
        <EvidenceModal
          title="Confirm fiat funding"
          label="Fiat receipt hash"
          onClose={() => setModal(null)}
          onSubmit={(hash) => {
            patchPayment(
              selected.id,
              {
                fundingStatus: FUNDING_STATUS.FIAT_CONFIRMED,
                fiatEvidenceHash: hash || nowHash(),
              },
              "Fiat funding confirmed"
            );
            setModal(null);
          }}
        />
      )}
      {modal?.type === "source" && (
        <AssignSettlerModal
          role="source"
          payment={selected}
          onClose={() => setModal(null)}
          onSubmit={(assignment) => {
            patchPayment(
              selected.id,
              (p) => ({
                ...p,
                status: PAYMENT_STATUS.SOURCE_ASSIGNED,
                settlers: { ...p.settlers, source: assignment },
              }),
              `Source settler assigned: ${assignment.settler.name}`
            );
            setModal(null);
          }}
        />
      )}
      {modal?.type === "dest" && (
        <AssignSettlerModal
          role="destination"
          payment={selected}
          onClose={() => setModal(null)}
          onSubmit={(assignment) => {
            patchPayment(
              selected.id,
              (p) => ({
                ...p,
                status: PAYMENT_STATUS.DEST_ASSIGNED,
                settlers: { ...p.settlers, destination: assignment },
              }),
              `Destination settler assigned: ${assignment.settler.name}`
            );
            setModal(null);
          }}
        />
      )}
      {modal?.type === "settlement" && (
        <EvidenceModal
          title="Confirm destination settlement"
          label="Settlement proof hash"
          onClose={() => setModal(null)}
          onSubmit={(hash) => {
            patchPayment(
              selected.id,
              (p) => ({
                ...p,
                status: PAYMENT_STATUS.READY_FOR_CLAIM,
                settlementEvidenceHash: hash || nowHash(),
                settlers: {
                  ...p.settlers,
                  destination: {
                    ...p.settlers.destination,
                    confirmed: true,
                    proofHash: hash || nowHash(),
                  },
                },
              }),
              "Destination settlement confirmed"
            );
            setModal(null);
          }}
        />
      )}
      {toast && (
        <div className="fixed bottom-5 right-5 z-50 rounded-2xl bg-slate-950 px-4 py-3 text-sm font-bold text-white shadow-soft">
          {toast}
        </div>
      )}
    </div>
  );
}

function Topbar({ view, setView, currentUser, setUserId, onCreate }) {
  return (
    <header className="sticky top-0 z-40 border-b border-slate-200 bg-white/95 backdrop-blur">
      <div className="mx-auto flex max-w-[1500px] items-center justify-between gap-4 px-4 py-4">
        <div className="flex items-center gap-3">
          <div className="grid h-11 w-11 place-items-center rounded-2xl bg-[#2F0FD1] text-white">
            <Coins className="h-5 w-5" />
          </div>
          <div>
            <p className="text-xs font-black tracking-[0.2em] text-slate-400 uppercase">
              Oynk Protocol
            </p>
            <h1 className="text-lg font-bold text-slate-950">Settlement MVP</h1>
          </div>
        </div>
        <div className="hidden rounded-2xl border border-slate-200 bg-slate-50 p-1 md:flex">
          {["user", "admin", "settler"].map((tab) => (
            <button
              key={tab}
              onClick={() => setView(tab)}
              className={cx(
                "rounded-xl px-4 py-2 text-sm font-bold capitalize",
                view === tab
                  ? "bg-white text-[#2F0FD1] shadow-sm"
                  : "text-slate-500"
              )}
            >
              {tab}
            </button>
          ))}
        </div>
        <div className="flex items-center gap-2">
          {view === "user" && (
            <select
              value={currentUser.id}
              onChange={(e) => setUserId(e.target.value)}
              className="hidden h-10 rounded-xl border border-slate-200 bg-white px-3 text-sm font-bold md:block"
            >
              {USERS.map((u) => (
                <option key={u.id} value={u.id}>
                  {u.name}
                </option>
              ))}
            </select>
          )}
          <button
            onClick={onCreate}
            className="inline-flex items-center gap-2 rounded-xl bg-[#2F0FD1] px-4 py-2.5 text-sm font-bold text-white hover:bg-[#2810B8]"
          >
            <Plus className="h-4 w-4" /> New payment
          </button>
        </div>
      </div>
      <div className="grid grid-cols-3 border-t border-slate-100 md:hidden">
        {["user", "admin", "settler"].map((tab) => (
          <button
            key={tab}
            onClick={() => setView(tab)}
            className={cx(
              "py-3 text-sm font-bold capitalize",
              view === tab ? "text-[#2F0FD1]" : "text-slate-500"
            )}
          >
            {tab}
          </button>
        ))}
      </div>
    </header>
  );
}

function Summary({ payments }) {
  const ready = payments.filter(
    (p) => p.status === PAYMENT_STATUS.READY_FOR_CLAIM
  ).length;
  const active = payments.filter(
    (p) =>
      ![
        PAYMENT_STATUS.COMPLETED,
        PAYMENT_STATUS.REFUNDED,
        PAYMENT_STATUS.CANCELLED,
      ].includes(p.status)
  ).length;
  const volume = payments.reduce((s, p) => s + Number(p.escrowAmount || 0), 0);
  return (
    <div className="grid gap-3 sm:grid-cols-3 lg:grid-cols-1">
      <Kpi label="Active payments" value={active} icon={Activity} />
      <Kpi label="Ready for claim" value={ready} icon={CheckCircle2} />
      <Kpi label="Quoted USDC" value={money(volume)} icon={Wallet} />
    </div>
  );
}

function Kpi({ label, value, icon: Icon }) {
  return (
    <div className="rounded-3xl border border-slate-200 bg-white p-4 shadow-sm">
      <div className="flex items-center gap-3">
        <div className="grid h-10 w-10 place-items-center rounded-2xl bg-indigo-50 text-[#2F0FD1]">
          <Icon className="h-5 w-5" />
        </div>
        <div>
          <p className="text-xs font-bold text-slate-500">{label}</p>
          <p className="text-xl font-black text-slate-950">{value}</p>
        </div>
      </div>
    </div>
  );
}

function PaymentList({
  payments,
  selectedId,
  setSelectedId,
  view,
  currentUser,
}) {
  const [q, setQ] = useState("");
  const filtered = payments.filter((p) => {
    const text = `${p.id} ${p.creator.name} ${p.paymentKind} ${currency(
      p.route.origin
    )} ${currency(p.route.destination)} ${p.status}`.toLowerCase();
    return text.includes(q.toLowerCase());
  });
  return (
    <div className="overflow-hidden rounded-3xl border border-slate-200 bg-white shadow-sm">
      <div className="border-b border-slate-200 p-4">
        <div className="relative">
          <Search className="absolute left-3 top-2.5 h-4 w-4 text-slate-400" />
          <input
            value={q}
            onChange={(e) => setQ(e.target.value)}
            placeholder="Search payments"
            className="h-10 w-full rounded-xl border border-slate-200 pl-9 pr-3 text-sm outline-none focus:border-[#2F0FD1]"
          />
        </div>
      </div>
      <div className="max-h-[680px] overflow-y-auto">
        {filtered.map((p) => (
          <button
            key={p.id}
            onClick={() => setSelectedId(p.id)}
            className={cx(
              "w-full border-b border-slate-100 p-4 text-left hover:bg-slate-50",
              selectedId === p.id && "bg-indigo-50/60"
            )}
          >
            <div className="flex items-start justify-between gap-3">
              <div>
                <p className="font-black text-slate-950">Payment #{p.id}</p>
                <p className="mt-1 text-xs font-semibold text-slate-500">
                  {p.creator.name} · {shortKind(p.paymentKind)}
                </p>
              </div>
              <Badge tone={paymentTone(p.status)}>
                {statusLabel(p.status)}
              </Badge>
            </div>
            <div className="mt-3 flex items-center gap-2 text-sm font-bold text-slate-700">
              <span>{currency(p.route.origin)}</span>
              <ArrowRight className="h-4 w-4 text-slate-400" />
              <span>{currency(p.route.destination)}</span>
            </div>
            <div className="mt-3 flex justify-between text-xs text-slate-500">
              <span>Destination</span>
              <span className="font-bold">
                {money(p.destinationAmount, p.route.destination || "USDC")}
              </span>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}

function UserPaymentView({ payment, currentUser, patchPayment }) {
  const isCreator = payment.creator.id === currentUser.id;

  const canCreatorDeposit =
    isCreator &&
    payment.paymentKind === PAYMENT_KIND.CRYPTO_TO_FIAT &&
    payment.fundingStatus === FUNDING_STATUS.QUOTE_SET &&
    payment.status === PAYMENT_STATUS.CREATED;

  const canCreatorClaim =
    isCreator &&
    payment.paymentKind === PAYMENT_KIND.FIAT_TO_CRYPTO &&
    payment.status === PAYMENT_STATUS.READY_FOR_CLAIM;

  return (
    <div className="space-y-6">
      <Hero payment={payment} />

      <div className="grid gap-6 xl:grid-cols-[1.1fr_0.9fr]">
        <Panel
          title="Payment progress"
          subtitle="The live operational state for this payment."
        >
          <FlowTimeline payment={payment} />
        </Panel>

        <Panel
          title="User actions"
          subtitle="Creator actions appear here when the contract state allows them."
        >
          <div className="space-y-3">
            {canCreatorDeposit ? (
              <PrimaryAction
                icon={Wallet}
                title="Deposit USDC escrow"
                desc={`Deposit ${money(
                  payment.escrowAmount
                )} into the protocol escrow.`}
                onClick={() =>
                  patchPayment(
                    payment.id,
                    {
                      fundingStatus: FUNDING_STATUS.READY,
                      status: PAYMENT_STATUS.CREATED,
                    },
                    "Creator deposited USDC escrow"
                  )
                }
              />
            ) : null}

            {canCreatorClaim ? (
              <PrimaryAction
                icon={Coins}
                title="Claim crypto"
                desc={`Claim ${money(payment.escrowAmount)} to your wallet.`}
                onClick={() =>
                  patchPayment(
                    payment.id,
                    { status: PAYMENT_STATUS.COMPLETED },
                    "Creator claimed crypto"
                  )
                }
              />
            ) : null}

            {!isCreator ? (
              <EmptyMessage
                title="Viewing another user's payment"
                text="Switch to the payment creator from the top bar to see creator actions."
              />
            ) : null}

            {isCreator && !canCreatorDeposit && !canCreatorClaim ? (
              <EmptyMessage
                title="No creator action available"
                text="This payment is waiting for a quote, fiat confirmation, settler action, or settlement processing."
              />
            ) : null}
          </div>
        </Panel>
      </div>

      <DetailsGrid payment={payment} />
    </div>
  );
}

function AdminPaymentView({ payment, setModal, patchPayment }) {
  const canSetQuote =
    payment.fundingStatus === FUNDING_STATUS.PENDING_QUOTE &&
    payment.status === PAYMENT_STATUS.CREATED;
  const canConfirmFiat =
    payment.paymentKind !== PAYMENT_KIND.CRYPTO_TO_FIAT &&
    payment.fundingStatus === FUNDING_STATUS.QUOTE_SET;
  const canAssignSource =
    payment.paymentKind !== PAYMENT_KIND.CRYPTO_TO_FIAT &&
    payment.fundingStatus === FUNDING_STATUS.FIAT_CONFIRMED &&
    !payment.settlers.source;
  const canAssignDest =
    payment.paymentKind !== PAYMENT_KIND.FIAT_TO_CRYPTO &&
    payment.fundingStatus === FUNDING_STATUS.READY &&
    !payment.settlers.destination &&
    payment.status === PAYMENT_STATUS.CREATED;
  return (
    <div className="space-y-6">
      <Hero payment={payment} />
      <div className="grid gap-6 xl:grid-cols-[1.1fr_0.9fr]">
        <Panel
          title="Broker admin command center"
          subtitle="Advance the payment by submitting the exact values used by the contract."
        >
          <div className="grid gap-3 md:grid-cols-2">
            <CommandButton
              disabled={!canSetQuote}
              icon={ReceiptText}
              title="Set quote"
              text="Enter source amount, escrow amount, and quote evidence hash."
              onClick={() => setModal({ type: "quote" })}
            />
            <CommandButton
              disabled={!canConfirmFiat}
              icon={Banknote}
              title="Confirm fiat funding"
              text="Submit fiat receipt hash after off-chain payment is verified."
              onClick={() => setModal({ type: "fiat" })}
            />
            <CommandButton
              disabled={!canAssignSource}
              icon={UsersRound}
              title="Assign source settler"
              text="Assign who deposits USDC escrow for fiat-funded orders."
              onClick={() => setModal({ type: "source" })}
            />
            <CommandButton
              disabled={!canAssignDest}
              icon={Building2}
              title="Assign destination settler"
              text="Assign who pays destination fiat and claims USDC."
              onClick={() => setModal({ type: "dest" })}
            />
          </div>
        </Panel>
        <Panel
          title="State machine"
          subtitle="Aligned to the latest Soroban contract."
        >
          <FlowTimeline payment={payment} compact />
        </Panel>
      </div>
      <DetailsGrid payment={payment} />
      <Panel
        title="Audit trail"
        subtitle="Operational history for this payment."
      >
        <div className="space-y-3">
          {payment.audit.map((a, i) => (
            <div
              key={i}
              className="flex items-center gap-3 rounded-2xl bg-slate-50 p-3 text-sm font-semibold text-slate-700"
            >
              <Clock3 className="h-4 w-4 text-slate-400" />
              {a}
            </div>
          ))}
        </div>
      </Panel>
    </div>
  );
}

function SettlerView({ payment, setModal, patchPayment }) {
  const source = payment.settlers.source;
  const dest = payment.settlers.destination;
  const canSourceDeposit =
    source &&
    !source.confirmed &&
    payment.status === PAYMENT_STATUS.SOURCE_ASSIGNED;
  const canDestAccept =
    dest && !dest.accepted && payment.status === PAYMENT_STATUS.DEST_ASSIGNED;
  const canDestConfirm =
    dest?.accepted &&
    !dest?.confirmed &&
    payment.status === PAYMENT_STATUS.SETTLEMENT_IN_PROGRESS;
  const canClaim =
    dest?.confirmed &&
    payment.status === PAYMENT_STATUS.READY_FOR_CLAIM &&
    payment.paymentKind !== PAYMENT_KIND.FIAT_TO_CRYPTO;
  return (
    <div className="space-y-6">
      <Hero payment={payment} />
      <Panel
        title="Settler workspace"
        subtitle="Work assigned source or destination settlement jobs."
      >
        <div className="grid gap-3 md:grid-cols-2">
          <CommandButton
            disabled={!canSourceDeposit}
            icon={Wallet}
            title="Deposit source escrow"
            text={
              source
                ? `Deposit ${money(payment.escrowAmount)} as source settler.`
                : "No source job assigned."
            }
            onClick={() => {
              patchPayment(
                payment.id,
                (p) => ({
                  ...p,
                  fundingStatus: FUNDING_STATUS.READY,
                  status:
                    p.paymentKind === PAYMENT_KIND.FIAT_TO_CRYPTO
                      ? PAYMENT_STATUS.READY_FOR_CLAIM
                      : PAYMENT_STATUS.CREATED,
                  settlers: {
                    ...p.settlers,
                    source: { ...p.settlers.source, confirmed: true },
                  },
                }),
                "Source settler deposited escrow"
              );
            }}
          />
          <CommandButton
            disabled={!canDestAccept}
            icon={Check}
            title="Accept destination job"
            text={
              dest
                ? `Accept payout of ${money(
                    dest.amountGives,
                    payment.route.destination
                  )}.`
                : "No destination job assigned."
            }
            onClick={() => {
              patchPayment(
                payment.id,
                (p) => ({
                  ...p,
                  status: PAYMENT_STATUS.SETTLEMENT_IN_PROGRESS,
                  settlers: {
                    ...p.settlers,
                    destination: { ...p.settlers.destination, accepted: true },
                  },
                }),
                "Destination settler accepted job"
              );
            }}
          />
          <CommandButton
            disabled={!canDestConfirm}
            icon={FileCheck2}
            title="Confirm fiat paid"
            text="Submit proof hash after destination fiat is released."
            onClick={() => setModal({ type: "settlement" })}
          />
          <CommandButton
            disabled={!canClaim}
            icon={Coins}
            title="Claim USDC payout"
            text={
              dest
                ? `Claim ${money(dest.amountReceives)} from escrow.`
                : "Settlement not ready."
            }
            onClick={() => {
              patchPayment(
                payment.id,
                { status: PAYMENT_STATUS.COMPLETED },
                "Destination settler claimed USDC payout"
              );
            }}
          />
        </div>
      </Panel>
      <DetailsGrid payment={payment} />
    </div>
  );
}

function Hero({ payment }) {
  return (
    <div className="overflow-hidden rounded-[2rem] border border-slate-200 bg-white shadow-sm">
      <div className="bg-slate-950 p-6 text-white">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <p className="text-xs font-black tracking-[0.18em] text-slate-400 uppercase">
              Payment #{payment.id}
            </p>
            <h2 className="mt-2 text-3xl font-black tracking-tight">
              {shortKind(payment.paymentKind)}
            </h2>
            <div className="mt-4 flex flex-wrap items-center gap-2">
              <Badge tone="blue">
                {currency(payment.route.origin)} →{" "}
                {currency(payment.route.destination)}
              </Badge>
              <Badge tone={fundingTone(payment.fundingStatus)}>
                {statusLabel(payment.fundingStatus)}
              </Badge>
              <Badge tone={paymentTone(payment.status)}>
                {statusLabel(payment.status)}
              </Badge>
            </div>
          </div>
          <div className="rounded-3xl bg-white/10 p-5 text-right">
            <p className="text-xs font-bold text-slate-300">
              Destination amount
            </p>
            <p className="mt-1 text-3xl font-black">
              {money(
                payment.destinationAmount,
                payment.route.destination || "USDC"
              )}
            </p>
            <p className="mt-2 text-sm font-semibold text-slate-300">
              Escrow:{" "}
              {payment.escrowAmount
                ? money(payment.escrowAmount)
                : "Awaiting quote"}
            </p>
          </div>
        </div>
      </div>
      <div className="grid gap-0 border-t border-slate-200 md:grid-cols-4">
        <HeroCell label="Creator" value={payment.creator.name} />
        <HeroCell
          label="Source amount"
          value={
            payment.sourceAmount
              ? money(payment.sourceAmount, payment.route.origin || "USDC")
              : "Not quoted"
          }
        />
        <HeroCell label="Created" value={payment.createdAt} />
        <HeroCell label="Deadline" value={payment.deadline} />
      </div>
    </div>
  );
}

function HeroCell({ label, value }) {
  return (
    <div className="border-b border-slate-100 p-4 md:border-b-0 md:border-r last:border-r-0">
      <p className="text-xs font-bold text-slate-400">{label}</p>
      <p className="mt-1 text-sm font-black text-slate-900">{value}</p>
    </div>
  );
}

function Panel({ title, subtitle, children }) {
  return (
    <section className="rounded-[1.75rem] border border-slate-200 bg-white p-5 shadow-sm">
      <div className="mb-5">
        <h3 className="text-lg font-black text-slate-950">{title}</h3>
        <p className="mt-1 text-sm text-slate-500">{subtitle}</p>
      </div>
      {children}
    </section>
  );
}

function FlowTimeline({ payment, compact }) {
  const steps = buildSteps(payment);
  return (
    <div className="space-y-4">
      {steps.map((s, i) => (
        <div key={s.label} className="flex gap-4">
          <div className="flex flex-col items-center">
            <div
              className={cx(
                "grid h-9 w-9 place-items-center rounded-full ring-1",
                s.done
                  ? "bg-emerald-50 text-emerald-700 ring-emerald-200"
                  : s.active
                  ? "bg-indigo-50 text-[#2F0FD1] ring-indigo-200"
                  : "bg-slate-50 text-slate-400 ring-slate-200"
              )}
            >
              {s.done ? <Check className="h-4 w-4" /> : i + 1}
            </div>
            {i < steps.length - 1 && (
              <div className="mt-2 h-8 w-px bg-slate-200" />
            )}
          </div>
          <div className="min-w-0 pb-2">
            <p className="font-black text-slate-900">{s.label}</p>
            <p className="mt-1 text-sm text-slate-500">{s.text}</p>
          </div>
        </div>
      ))}
    </div>
  );
}

function buildSteps(p) {
  const base = [
    {
      label: "Payment created",
      text: "User created a payment request.",
      done: true,
    },
    {
      label: "Quote set",
      text: "Admin sets source amount, USDC escrow amount, and quote evidence.",
      done: p.fundingStatus !== FUNDING_STATUS.PENDING_QUOTE,
      active: p.fundingStatus === FUNDING_STATUS.PENDING_QUOTE,
    },
  ];
  if (p.paymentKind !== PAYMENT_KIND.CRYPTO_TO_FIAT)
    base.push({
      label: "Fiat confirmed",
      text: "Admin confirms off-chain fiat payment.",
      done: [FUNDING_STATUS.FIAT_CONFIRMED, FUNDING_STATUS.READY].includes(
        p.fundingStatus
      ),
      active: p.fundingStatus === FUNDING_STATUS.QUOTE_SET,
    });
  if (p.paymentKind !== PAYMENT_KIND.CRYPTO_TO_FIAT)
    base.push({
      label: "Source escrow",
      text: "Source settler deposits USDC escrow.",
      done: p.fundingStatus === FUNDING_STATUS.READY,
      active: p.status === PAYMENT_STATUS.SOURCE_ASSIGNED,
    });
  else
    base.push({
      label: "Creator escrow",
      text: "Creator deposits USDC into protocol.",
      done: p.fundingStatus === FUNDING_STATUS.READY,
      active: p.fundingStatus === FUNDING_STATUS.QUOTE_SET,
    });
  if (p.paymentKind === PAYMENT_KIND.FIAT_TO_CRYPTO)
    base.push({
      label: "Creator claim",
      text: "Creator claims USDC.",
      done: p.status === PAYMENT_STATUS.COMPLETED,
      active: p.status === PAYMENT_STATUS.READY_FOR_CLAIM,
    });
  if (p.paymentKind !== PAYMENT_KIND.FIAT_TO_CRYPTO) {
    base.push({
      label: "Destination settler",
      text: "Admin assigns destination payout settler.",
      done: !!p.settlers.destination,
      active:
        p.fundingStatus === FUNDING_STATUS.READY && !p.settlers.destination,
    });
    base.push({
      label: "Fiat payout",
      text: "Destination settler pays recipient fiat and submits proof.",
      done:
        p.status === PAYMENT_STATUS.READY_FOR_CLAIM ||
        p.status === PAYMENT_STATUS.COMPLETED,
      active: p.status === PAYMENT_STATUS.SETTLEMENT_IN_PROGRESS,
    });
    base.push({
      label: "Settler claim",
      text: "Destination settler claims USDC payout.",
      done: p.status === PAYMENT_STATUS.COMPLETED,
      active: p.status === PAYMENT_STATUS.READY_FOR_CLAIM,
    });
  }
  return base;
}

function DetailsGrid({ payment }) {
  return (
    <div className="grid gap-6 xl:grid-cols-2">
      <Panel
        title="References and evidence"
        subtitle="Hashes map off-chain records to on-chain actions."
      >
        <InfoRow label="Sender ref" value={payment.senderRef} copy />
        <InfoRow label="Recipient ref" value={payment.recipientRef} copy />
        <InfoRow
          label="Quote evidence"
          value={payment.quoteEvidenceHash || "Not set"}
          copy={!!payment.quoteEvidenceHash}
        />
        <InfoRow
          label="Fiat evidence"
          value={payment.fiatEvidenceHash || "Not set"}
          copy={!!payment.fiatEvidenceHash}
        />
        <InfoRow
          label="Settlement evidence"
          value={payment.settlementEvidenceHash || "Not set"}
          copy={!!payment.settlementEvidenceHash}
        />
      </Panel>
      <Panel title="Settlers" subtitle="Fixed source and destination roles.">
        <SettlerCard
          title="Source settler"
          assignment={payment.settlers.source}
        />
        <SettlerCard
          title="Destination settler"
          assignment={payment.settlers.destination}
        />
      </Panel>
    </div>
  );
}

function InfoRow({ label, value, copy }) {
  const [copied, setCopied] = useState(false);
  return (
    <div className="flex items-center justify-between gap-4 border-b border-slate-100 py-3 last:border-b-0">
      <div className="min-w-0">
        <p className="text-xs font-bold text-slate-400">{label}</p>
        <p className="mt-1 truncate text-sm font-bold text-slate-800">
          {value}
        </p>
      </div>
      {copy && (
        <button
          onClick={async () => {
            await navigator.clipboard?.writeText(value);
            setCopied(true);
            setTimeout(() => setCopied(false), 1000);
          }}
          className="rounded-xl border border-slate-200 p-2 text-slate-500"
        >
          {copied ? (
            <CheckCircle2 className="h-4 w-4 text-emerald-600" />
          ) : (
            <Copy className="h-4 w-4" />
          )}
        </button>
      )}
    </div>
  );
}

function SettlerCard({ title, assignment }) {
  if (!assignment)
    return (
      <div className="mb-3 rounded-2xl border border-dashed border-slate-200 p-4">
        <p className="font-black text-slate-700">{title}</p>
        <p className="mt-1 text-sm text-slate-500">Not assigned</p>
      </div>
    );
  return (
    <div className="mb-3 rounded-2xl border border-slate-200 bg-slate-50 p-4">
      <p className="font-black text-slate-900">{title}</p>
      <p className="mt-1 text-sm font-semibold text-slate-600">
        {assignment.settler.name}
      </p>
      <div className="mt-3 flex flex-wrap gap-2">
        <Badge tone={assignment.accepted ? "green" : "slate"}>
          {assignment.accepted ? "Accepted" : "Not accepted"}
        </Badge>
        <Badge tone={assignment.confirmed ? "green" : "amber"}>
          {assignment.confirmed ? "Confirmed" : "Pending"}
        </Badge>
      </div>
      <p className="mt-3 text-xs text-slate-500">
        Gives {money(assignment.amountGives)} · Receives{" "}
        {money(assignment.amountReceives)}
      </p>
    </div>
  );
}

function CommandButton({ disabled, icon: Icon, title, text, onClick }) {
  return (
    <button
      disabled={disabled}
      onClick={onClick}
      className="rounded-2xl border border-slate-200 p-4 text-left transition hover:border-[#2F0FD1] hover:bg-indigo-50/40 disabled:cursor-not-allowed disabled:opacity-45 disabled:hover:border-slate-200 disabled:hover:bg-white"
    >
      <div className="flex gap-3">
        <div className="grid h-10 w-10 shrink-0 place-items-center rounded-xl bg-indigo-50 text-[#2F0FD1]">
          <Icon className="h-5 w-5" />
        </div>
        <div>
          <p className="font-black text-slate-950">{title}</p>
          <p className="mt-1 text-sm leading-5 text-slate-500">{text}</p>
        </div>
      </div>
    </button>
  );
}

function PrimaryAction({ icon: Icon, title, desc, onClick, disabled }) {
  return (
    <button
      disabled={disabled}
      onClick={onClick}
      className="flex w-full items-center gap-3 rounded-2xl bg-slate-950 p-4 text-left text-white disabled:opacity-50"
    >
      <Icon className="h-5 w-5" />
      <div>
        <p className="font-black">{title}</p>
        <p className="text-sm text-slate-300">{desc}</p>
      </div>
    </button>
  );
}

function EmptyMessage({ title, text }) {
  return (
    <div className="rounded-2xl border border-dashed border-slate-200 bg-slate-50 p-5 text-center">
      <Info className="mx-auto h-5 w-5 text-slate-400" />
      <p className="mt-2 font-black text-slate-800">{title}</p>
      <p className="mt-1 text-sm text-slate-500">{text}</p>
    </div>
  );
}

function CreatePaymentModal({ onClose, onSubmit }) {
  const [form, setForm] = useState({
    paymentKind: PAYMENT_KIND.FIAT_TO_FIAT,
    origin: 566,
    destination: 840,
    destinationAmount: 1000,
  });
  const kind = form.paymentKind;
  const canSubmit =
    Number(form.destinationAmount) > 0 &&
    (kind === PAYMENT_KIND.CRYPTO_TO_FIAT || form.origin) &&
    (kind === PAYMENT_KIND.FIAT_TO_CRYPTO || form.destination);
  return (
    <Modal
      title="Create payment"
      subtitle="Create the exact request that maps to create_payment()."
      onClose={onClose}
    >
      <div className="space-y-4">
        <Field label="Payment type">
          <select
            value={form.paymentKind}
            onChange={(e) => setForm({ ...form, paymentKind: e.target.value })}
            className="input"
          >
            <option value={PAYMENT_KIND.FIAT_TO_CRYPTO}>Fiat to Crypto</option>
            <option value={PAYMENT_KIND.CRYPTO_TO_FIAT}>Crypto to Fiat</option>
            <option value={PAYMENT_KIND.FIAT_TO_FIAT}>Fiat to Fiat</option>
          </select>
        </Field>
        <div className="grid gap-3 md:grid-cols-2">
          {kind !== PAYMENT_KIND.CRYPTO_TO_FIAT && (
            <CurrencyField
              label="Origin fiat"
              value={form.origin}
              onChange={(v) => setForm({ ...form, origin: v })}
            />
          )}
          {kind !== PAYMENT_KIND.FIAT_TO_CRYPTO && (
            <CurrencyField
              label="Destination fiat"
              value={form.destination}
              onChange={(v) => setForm({ ...form, destination: v })}
            />
          )}
        </div>
        <Field label="Destination amount">
          <input
            className="input"
            type="number"
            value={form.destinationAmount}
            onChange={(e) =>
              setForm({ ...form, destinationAmount: e.target.value })
            }
          />
        </Field>
        <ModalActions
          onClose={onClose}
          disabled={!canSubmit}
          submitLabel="Create payment"
          onSubmit={() => onSubmit(form)}
        />
      </div>
    </Modal>
  );
}

function QuoteModal({ payment, onClose, onSubmit }) {
  const [values, setValues] = useState({
    sourceAmount: payment.sourceAmount || "",
    escrowAmount: payment.escrowAmount || "",
    evidenceHash: nowHash(),
  });
  const valid =
    Number(values.sourceAmount) > 0 &&
    Number(values.escrowAmount) > 0 &&
    values.evidenceHash.length >= 8;
  return (
    <Modal
      title="Set quote"
      subtitle="Enter the quote values. This mirrors set_quote(source_amount, escrow_amount, evidence_hash)."
      onClose={onClose}
    >
      <div className="space-y-4">
        <Field
          label={`Source amount (${currency(payment.route.origin || "USDC")})`}
        >
          <input
            className="input"
            type="number"
            value={values.sourceAmount}
            onChange={(e) =>
              setValues({ ...values, sourceAmount: e.target.value })
            }
          />
        </Field>
        <Field label="USDC escrow amount">
          <input
            className="input"
            type="number"
            value={values.escrowAmount}
            onChange={(e) =>
              setValues({ ...values, escrowAmount: e.target.value })
            }
          />
        </Field>
        <Field label="Quote evidence hash">
          <input
            className="input"
            value={values.evidenceHash}
            onChange={(e) =>
              setValues({ ...values, evidenceHash: e.target.value })
            }
          />
        </Field>
        <ModalActions
          onClose={onClose}
          disabled={!valid}
          submitLabel="Set quote"
          onSubmit={() => onSubmit(values)}
        />
      </div>
    </Modal>
  );
}

function EvidenceModal({ title, label, onClose, onSubmit }) {
  const [hash, setHash] = useState(nowHash());
  return (
    <Modal
      title={title}
      subtitle="Submit the evidence hash used for audit and verification."
      onClose={onClose}
    >
      <div className="space-y-4">
        <Field label={label}>
          <input
            className="input"
            value={hash}
            onChange={(e) => setHash(e.target.value)}
          />
        </Field>
        <ModalActions
          onClose={onClose}
          disabled={hash.trim().length < 8}
          submitLabel="Confirm"
          onSubmit={() => onSubmit(hash.trim())}
        />
      </div>
    </Modal>
  );
}

function AssignSettlerModal({ role, payment, onClose, onSubmit }) {
  const eligible = SETTLERS.filter((s) =>
    role === "source" ? s.type === "source" : s.type === "destination"
  );
  const [settlerId, setSettlerId] = useState(eligible[0]?.id);
  const [amountGives, setAmountGives] = useState(
    role === "source"
      ? payment.escrowAmount || ""
      : payment.destinationAmount || ""
  );
  const [amountReceives, setAmountReceives] = useState(
    role === "source" ? payment.sourceAmount || "" : payment.escrowAmount || ""
  );
  const valid =
    settlerId && Number(amountGives) > 0 && Number(amountReceives) > 0;
  return (
    <Modal
      title={`Assign ${role} settler`}
      subtitle="Enter assignment values exactly as they will be stored."
      onClose={onClose}
    >
      <div className="space-y-4">
        <Field label="Settler">
          <select
            className="input"
            value={settlerId}
            onChange={(e) => setSettlerId(e.target.value)}
          >
            {eligible.map((s) => (
              <option key={s.id} value={s.id}>
                {s.name} · {s.region}
              </option>
            ))}
          </select>
        </Field>
        <div className="grid gap-3 md:grid-cols-2">
          <Field label="Amount gives">
            <input
              className="input"
              type="number"
              value={amountGives}
              onChange={(e) => setAmountGives(e.target.value)}
            />
          </Field>
          <Field label="Amount receives">
            <input
              className="input"
              type="number"
              value={amountReceives}
              onChange={(e) => setAmountReceives(e.target.value)}
            />
          </Field>
        </div>
        <ModalActions
          onClose={onClose}
          disabled={!valid}
          submitLabel="Assign settler"
          onSubmit={() =>
            onSubmit({
              settler: eligible.find((s) => s.id === settlerId),
              amountGives: Number(amountGives),
              amountReceives: Number(amountReceives),
              accepted: false,
              confirmed: false,
              proofHash: null,
            })
          }
        />
      </div>
    </Modal>
  );
}

function CurrencyField({ label, value, onChange }) {
  return (
    <Field label={label}>
      <select
        className="input"
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
      >
        {CURRENCIES.map((c) => (
          <option key={c.code} value={c.code}>
            {c.symbol} · {c.name}
          </option>
        ))}
      </select>
    </Field>
  );
}

function Field({ label, children }) {
  return (
    <label className="block">
      <span className="mb-2 block text-sm font-black text-slate-800">
        {label}
      </span>
      {children}
    </label>
  );
}

function Modal({ title, subtitle, onClose, children }) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/50 p-4 backdrop-blur-sm">
      <div className="w-full max-w-xl rounded-[2rem] bg-white p-6 shadow-soft">
        <div className="mb-5 flex items-start justify-between gap-4">
          <div>
            <h3 className="text-2xl font-black text-slate-950">{title}</h3>
            <p className="mt-1 text-sm leading-6 text-slate-500">{subtitle}</p>
          </div>
          <button
            onClick={onClose}
            className="rounded-xl border border-slate-200 p-2 text-slate-500"
          >
            <X className="h-5 w-5" />
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}

function ModalActions({ onClose, disabled, submitLabel, onSubmit }) {
  return (
    <div className="grid grid-cols-2 gap-3 pt-2">
      <button
        onClick={onClose}
        className="rounded-xl border border-slate-200 px-4 py-3 font-black text-slate-700 hover:bg-slate-50"
      >
        Cancel
      </button>
      <button
        disabled={disabled}
        onClick={onSubmit}
        className="rounded-xl bg-[#2F0FD1] px-4 py-3 font-black text-white hover:bg-[#2810B8] disabled:bg-slate-300"
      >
        {" "}
        {submitLabel}
      </button>
    </div>
  );
}

const style = document.createElement("style");
style.innerHTML = `.input{height:44px;width:100%;border-radius:14px;border:1px solid #e2e8f0;background:white;padding:0 12px;font-size:14px;font-weight:700;color:#0f172a;outline:none}.input:focus{border-color:#2F0FD1;box-shadow:0 0 0 3px rgba(47,15,209,.08)}`;
document.head.appendChild(style);

createRoot(document.getElementById("root")).render(<App />);
