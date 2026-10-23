"""Post-INTENT lifetime tests using historical signatures and public fakes.

Retained full-host ELF/account/message seams remain explicit. No real network,
private input, signer, signing, simulation, send or current-chain observation.
"""
import importlib.util
import json
from pathlib import Path
import unittest
from unittest import mock

HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location("send_lifetime_helpers",HERE/"test_startup_lifetime_capture.py")
t=importlib.util.module_from_spec(spec);spec.loader.exec_module(t);o=t.o


class SendLifetimeGuardTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):t.LifetimeCaptureTests.setUpClass()

    def test_expiry_invalid_hash_and_auth_after_durable_intent_hold_without_send(self):
        for fault,reason in (("height","post_intent_lifetime_margin"),
                             ("invalid","post_intent_blockhash_invalid_or_stale"),
                             ("authorization","launcher_authorization_expired")):
            with self.subTest(fault=fault):
                e=t.Environment(self)
                try:
                    e.capture();e.signed();auth=e.host.auth();clock=[1]
                    append=o.Journal.append
                    def delayed(instance,kind,data):
                        record=append(instance,kind,data)
                        if kind=="INTENT":
                            self.assertEqual(e.records()[-1]["kind"],"INTENT")
                            if fault=="height":e.send_height=961
                            elif fault=="invalid":e.fault="invalid"
                            else:clock[0]=18446744073709551616
                        return record
                    with mock.patch.object(o.Journal,"append",delayed),mock.patch.object(o.time,"time",side_effect=lambda:clock[0]):
                        with self.assertRaisesRegex(o.Stop,reason):
                            e.host.call("SEND",{"id":e.pid},_transport=e.transport,**auth)
                        self.assertFalse(e.sent)
                        records=e.records();self.assertEqual(records[-1]["kind"],"INTENT")
                        self.assertEqual(sum(r["kind"]=="INTENT" for r in records),1)
                        self.assertFalse(any(r["kind"] in ("SUBMITTED","UNKNOWN") for r in records))
                        e.host.refresh_tail();report=e.host.call("INSPECT")
                        self.assertEqual(report["status"],"UNKNOWN_OR_SUBMITTED_RECOVERY_ONLY")
                        self.assertEqual(report["proposedFeeLedger"]["reservedFeeLamports"],"10000")
                        clock[0]=1;before=len(e.queries)
                        with self.assertRaisesRegex(o.Stop,"intent_exists_recovery_only"):
                            e.host.call("SEND",{"id":e.pid},_transport=e.transport,**auth)
                        self.assertEqual(len(e.queries),before)
                        with self.assertRaisesRegex(o.Stop,"unresolved_intent_blocks_prepare"):e.capture()
                        self.assertEqual(len(e.queries),before)
                finally:e.full.doCleanups()

    def test_full_fourteen_rents_and_post_intent_quote_fit_unchanged_six_call_budget(self):
        e=t.Environment(self);e.capture();e.signed();e.queries.clear();e.calls.clear()
        transport=e.transport;events=[];post_floor=e.slot-1
        def observed(raw):
            queries=json.loads(raw);methods=[q["method"] for q in queries]
            if methods==["getBlockHeight","isBlockhashValid","getLatestBlockhash"]:
                self.assertEqual(e.records()[-1]["kind"],"INTENT");events.append("post-intent-quote")
                self.assertEqual(queries[1]["params"][0],e.original)
                for q in queries:
                    self.assertGreaterEqual(q["params"][-1]["minContextSlot"],post_floor)
            if methods==["sendTransaction"]:
                self.assertEqual(events,["post-intent-quote"]);events.append("send")
                self.assertGreaterEqual(queries[0]["params"][1]["minContextSlot"],post_floor)
                self.assertEqual(queries[0]["params"][1]["maxRetries"],0)
                self.assertFalse(queries[0]["params"][1]["skipPreflight"])
            return transport(raw)
        report=e.host.call("SEND",{"id":e.pid},_transport=observed,**e.host.auth())
        self.assertTrue(e.sent);self.assertEqual(events,["post-intent-quote","send"])
        self.assertEqual([len(batch) for batch in e.queries],[5,5,5,5,3,1])
        self.assertEqual([q["method"] for q in e.queries[0]][:2],["getGenesisHash","getVersion"])
        rent_sizes=[q["params"][0] for batch in e.queries for q in batch if q["method"]=="getMinimumBalanceForRentExemption"]
        self.assertEqual(rent_sizes,sorted(map(int,e.ref["rents"])))
        self.assertEqual(len(rent_sizes),14);self.assertEqual(e.calls.count("sendTransaction"),1)
        state=o.replay(e.host.j,e.records())
        self.assertEqual(state["prepared"][e.pid]["lifetime"]["lastValidBlockHeight"],"1000")
        self.assertEqual(state["prepared"][e.pid]["lifetime"]["minimumRemainingBlockHeights"],"40")
        self.assertEqual(report["status"],"UNKNOWN_OR_SUBMITTED_RECOVERY_ONLY")
        self.assertFalse(report["liveReady"]);self.assertFalse(report["signerInvoked"])


if __name__=="__main__":unittest.main()
