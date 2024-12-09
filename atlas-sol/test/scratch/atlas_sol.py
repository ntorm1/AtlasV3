import json
import os, sys
import requests
from dataclasses import dataclass
import asyncio
from typing import List
from pythclient.pythaccounts import PythPriceAccount, PythPriceStatus
from pythclient.solana import (
    SolanaClient,
    SolanaPublicKey,
    PYTHNET_HTTP_ENDPOINT,
    PYTHNET_WS_ENDPOINT,
)
from pythclient.pythclient import PythClient  # noqa
from pythclient.utils import get_key  # noqa

from solana.rpc.api import Client, Signature
from solders.pubkey import Pubkey
from solders.rpc.responses import GetBlockResp, GetTransactionResp
from solders.transaction_status import (
    UiConfirmedBlock,
    EncodedTransactionWithStatusMeta,
)
from typing import List, Any, Dict, Optional, Union
from solders.signature import Signature
import numpy as np
import datetime as dt
import polars as pl
from sqlalchemy import create_engine

from atlas_enum import AtlasCryptoCol


# ========================================================================
@dataclass
class SolTransaction:
    owner: str
    account: str
    mint: str
    wallet_prebalance: float
    wallet_postbalance: float
    owner_cp: str
    account_cp: str
    wallet_prebalance_cp: float
    wallet_postbalance_cp: float
    mint_cp: str
    wallet_prebalance_sold: float
    wallet_postbalance_sold: float
    account_sold: str

    # ========================================================================
    def __init__(self, **kwargs):
        self.signature = kwargs["signature"]
        self.owner = kwargs["owner"]
        self.account = kwargs["account"]
        self.mint = kwargs["mint"]
        self.wallet_prebalance = kwargs["wallet_prebalance"]
        self.wallet_postbalance = kwargs["wallet_postbalance"]
        self.owner_cp = kwargs.get("owner_cp", None)
        self.owner_cp = kwargs.get("owner_cp", None)
        self.account_cp = kwargs.get("account_cp", None)
        self.wallet_prebalance_cp = kwargs.get("wallet_prebalance_cp", None)
        self.wallet_postbalance_cp = kwargs.get("wallet_postbalance_cp", None)
        self.mint_cp = kwargs.get("mint_cp", None)
        self.wallet_prebalance_sold = kwargs.get("wallet_prebalance_sold", None)
        self.wallet_postbalance_sold = kwargs.get("wallet_postbalance_sold", None)
        self.account_sold = kwargs.get("account_sold", None)


# ========================================================================
class AtlasSol:

    # ========================================================================
    def __init__(self, url: str = "https://api.mainnet-beta.solana.com") -> None:
        self.client = Client(url)

    # ========================================================================
    @staticmethod
    def GetPostgresEngine() -> Any:
        conn_str = "postgresql://postgres:postgres@localhost:5432/solana"
        return create_engine(conn_str)

    # ========================================================================
    def getTxFrame(
        self, block: Union[GetBlockResp, UiConfirmedBlock], raise_on_error: bool = True
    ) -> pl.DataFrame:
        t_array = []
        if isinstance(block, UiConfirmedBlock):
            transactions = block.transactions
            block_time = block.block_time
            prt_slot = block.parent_slot
        else:
            transactions = block.value.transactions
            block_time = block.value.block_time
            prt_slot = block.value.parent_slot
        for idx, tx in enumerate(transactions):
            try:
                tx_res = self.getTransaction(tx)
                if tx_res is None:
                    continue
                t_array.append(tx_res)
            except Exception:
                if raise_on_error:
                    raise RuntimeError(
                        f"Error in transaction {idx}: {tx}\n{sys.exc_info()}"
                    )
                continue
        return pl.DataFrame([t.__dict__ for t in t_array]).with_columns(
            pl.lit(dt.datetime.fromtimestamp(block_time, tz=dt.timezone.utc)).alias(
                AtlasCryptoCol.Timestamp.value
            ),
            pl.lit(prt_slot).cast(pl.Int64).alias(AtlasCryptoCol.ParentSlot.value),
        )

    # ========================================================================
    @staticmethod
    def GetMintkeys(block_frame: pl.DataFrame) -> List[str]:
        res_all = []
        for col in [AtlasCryptoCol.Mint.value, AtlasCryptoCol.MintCp.value]:
            res_all.extend(
                block_frame.select(pl.col(col).unique()).to_numpy().flatten().tolist()
            )
        return list(set(res_all))

    # ========================================================================
    def getTransaction(
        self, signature: Union[str, Signature, EncodedTransactionWithStatusMeta]
    ) -> Optional[SolTransaction]:
        if isinstance(signature, Signature):
            signature = signature.__str__()
        if isinstance(signature, EncodedTransactionWithStatusMeta):
            if signature.meta.err is not None:
                return None
            return self._getTransaction(self._getTransactionComps(signature))
        transaction = self.client.get_transaction(
            tx_sig=Signature.from_string(signature),
            max_supported_transaction_version=0,
        )
        return self._getTransaction(self._getTransactionComps(transaction))

    # ========================================================================
    def _getTransaction(self, tx_comp: List[Any]) -> Optional[SolTransaction]:
        if len(tx_comp) == 0:
            return None
        tx_cp_found = False
        tx_idx = 0
        while not tx_cp_found and tx_idx < len(tx_comp):
            tx_this = tx_comp[tx_idx]
            tx_this_amount = tx_this["diff"]
            min_idx = -1
            min_val = sys.float_info.max
            for tx_idx_cp, tx_cp in enumerate(tx_comp):
                tx_sum = abs(tx_this_amount + tx_cp["diff"])
                if (
                    tx_sum < min_val
                    and tx_cp["owner"] != tx_this["owner"]
                    and tx_sum < 1e-5
                ):
                    min_val = tx_sum
                    min_idx = tx_idx_cp
                    break
            if min_idx != -1:

                def find_matching_tx(tx_comp, owner, diff_sign):
                    for tx_cp in tx_comp:
                        if (
                            tx_cp["owner"] == owner
                            and np.sign(tx_cp["diff"]) != diff_sign
                        ):
                            return tx_cp
                    return None

                tx_cp = tx_comp[tx_idx_cp]
                tx_this.update(
                    {
                        "owner_cp": tx_cp["owner"],
                        "account_cp": tx_cp["account"],
                        "wallet_prebalance_cp": tx_cp["wallet_prebalance"],
                        "wallet_postbalance_cp": tx_cp["wallet_postbalance"],
                    }
                )
                matching_tx = find_matching_tx(
                    tx_comp, tx_this["owner"], np.sign(tx_this["diff"])
                )
                if not matching_tx:
                    matching_tx = find_matching_tx(
                        tx_comp, tx_this["owner_cp"], np.sign(tx_this["diff"])
                    )
                if matching_tx:
                    tx_this.update(
                        {
                            "mint_cp": matching_tx["mint"],
                            "wallet_prebalance_sold": matching_tx["wallet_prebalance"],
                            "wallet_postbalance_sold": matching_tx[
                                "wallet_postbalance"
                            ],
                            "account_sold": matching_tx["account"],
                            "diff_sold": matching_tx["diff"],
                        }
                    )
            if "mint_cp" in tx_this:
                tx_cp_found = True
            tx_idx += 1
        return SolTransaction(**tx_this)

    # ========================================================================
    def _getTransactionComps(self, transaction: GetTransactionResp) -> List[Any]:
        tx_comp = []

        if not isinstance(transaction, GetTransactionResp):
            tx_meta = transaction.meta
            accounts = transaction.transaction.message.account_keys
            signature = transaction.transaction.signatures[0]
        else:
            tx_meta = transaction.value.transaction.meta
            accounts = transaction.value.transaction.transaction.message.account_keys
            signature = transaction.value.transaction.transaction.signatures[0]
        if (
            len(tx_meta.pre_token_balances) == 0
            and len(tx_meta.post_token_balances) == 0
        ):
            return tx_comp
        post_token_idx_set = set()
        for token_idx, pre_token_balance in enumerate(tx_meta.pre_token_balances):
            owner = pre_token_balance.owner
            account = None
            if pre_token_balance.account_index >= len(accounts):
                account_index_offset = pre_token_balance.account_index - len(accounts)
                if account_index_offset >= len(tx_meta.loaded_addresses.writable):
                    account_index_offset -= len(tx_meta.loaded_addresses.writable)
                    account = tx_meta.loaded_addresses.readonly[account_index_offset]
                else:
                    account = tx_meta.loaded_addresses.writable[account_index_offset]
            else:
                account = accounts[pre_token_balance.account_index]
            mint = pre_token_balance.mint.__str__()
            wallet_prebalance = float(
                pre_token_balance.ui_token_amount.ui_amount_string
            )
            post_token_idx = -1
            for idx, post_token_balance in enumerate(tx_meta.post_token_balances):
                if post_token_balance.account_index == pre_token_balance.account_index:
                    post_token_idx = idx
                    post_token_idx_set.add(post_token_idx)
                    break
            if post_token_idx == -1:
                wallet_postbalance = 0
            else:
                wallet_postbalance = float(
                    tx_meta.post_token_balances[
                        post_token_idx
                    ].ui_token_amount.ui_amount_string
                )
            if wallet_postbalance == wallet_prebalance:
                continue
            tx_comp.append(
                {
                    "signature": signature.__str__(),
                    "owner": owner.__str__(),
                    "account": account.__str__(),
                    "mint": mint,
                    "wallet_prebalance": wallet_prebalance,
                    "wallet_postbalance": wallet_postbalance,
                    "diff": wallet_postbalance - wallet_prebalance,
                }
            )
        if len(post_token_idx_set) < len(tx_meta.post_token_balances):
            for post_token_idx, post_token_balance in enumerate(
                tx_meta.post_token_balances
            ):
                if post_token_idx not in post_token_idx_set:
                    wallet_prebalance = 0
                    wallet_postbalance = float(
                        post_token_balance.ui_token_amount.ui_amount_string
                    )
                    tx_comp.append(
                        {
                            "signature": signature.__str__(),
                            "owner": post_token_balance.owner.__str__(),
                            "account": accounts[
                                post_token_balance.account_index
                            ].__str__(),
                            "mint": post_token_balance.mint.__str__(),
                            "wallet_prebalance": wallet_prebalance,
                            "wallet_postbalance": wallet_postbalance,
                            "diff": wallet_postbalance - wallet_prebalance,
                        }
                    )
        return tx_comp

    # ========================================================================
    def getTokenList() -> pl.DataFrame:
        TOKEN_LIST_URL = "https://raw.githubusercontent.com/solana-labs/token-list/main/src/tokens/solana.tokenlist.json"
        response = requests.get(TOKEN_LIST_URL)
        response.raise_for_status()
        token_list = response.json()
        return pl.DataFrame(token_list["tokens"])

    # ========================================================================
    async def getPrices(self) -> pl.DataFrame:
        v2_first_mapping_account_key = get_key("pythnet", "mapping")
        v2_program_key = get_key("pythnet", "program")
        res = []
        async with PythClient(
            first_mapping_account_key=v2_first_mapping_account_key,
            program_key=v2_program_key if False else None,
            solana_endpoint=PYTHNET_HTTP_ENDPOINT,  # replace with the relevant cluster endpoints
            solana_ws_endpoint=PYTHNET_WS_ENDPOINT,  # replace with the relevant cluster endpoints
        ) as c:
            await c.refresh_all_prices()
            products = await c.get_products()
            all_prices: List[PythPriceAccount] = []
            for p in products:
                # print(p.key, p.attrs)
                prices = await p.get_prices()
                for _, pr in prices.items():
                    all_prices.append(pr)
                    res.append(
                        {
                            "key": pr.key.__str__(),
                            "product_account_key": pr.product_account_key.__str__(),
                            "symbol": pr.product.attrs["display_symbol"],
                            "status": pr.aggregate_price_status.name,
                            "price": pr.aggregate_price,
                            "timestamp": pr.timestamp,
                            "ci": pr.aggregate_price_confidence_interval,
                        }
                    )
                if pr.product.attrs["display_symbol"] == "SOL/USD":
                    break
        return (
            pl.DataFrame(res)
            .filter(pl.col("price").is_not_null())
            .filter(pl.col("status") == "TRADING")
        )
