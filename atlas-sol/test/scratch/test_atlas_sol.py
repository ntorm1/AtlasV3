import unittest

from atlas_sol import AtlasSol, SolTransaction


# ========================================================================
class AtlasSolTest(unittest.TestCase):

    # ========================================================================
    def setUp(self):
        self.atlas_sol = AtlasSol()

    # ========================================================================
    def testGetTransactionSimple(self):
        tx = self.atlas_sol.getTransaction(
            signature="3boyDShobz2MhfCie975MdAcufzM2fbVDKjipRc7xmmWQhQQhHGtaHatVZgcYmqyMnkUDCGv85hNDRYNkkDCvJqK"
        )
        EXPECTED = {
            "owner": "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1",
            "account": "ViaWBq2QPE83wYCsv6Yhe16jEgPSxZmXTXBgZqSNJhX",
            "mint": "9PPeYELzXxnxTuWkX7P9JH9J9hsXc9b8uoGNJwA1pump",
            "wallet_prebalance": 293920207.516883,
            "wallet_postbalance": 297594312.850405,
            "diff": 3674105.333521962,
            "owner_cp": "J3g4GWCCcQkeZjzWu6Hn1VRj3Qouz4SwYKG84akxTdgS",
            "account_cp": "AKbRjX1vyvGCJqTQ5gqERJKvrzoJmFNb5xnAHx7MNyaq",
            "wallet_prebalance_cp": 3674105.333522,
            "wallet_postbalance_cp": 0,
            "mint_cp": "So11111111111111111111111111111111111111112",
            "wallet_prebalance_sold": 59.635501108,
            "wallet_postbalance_sold": 58.901058004,
            "account_sold": "E95KPBjkeLVy2to5ahJDnLA21dYRBq7c13L916cuobgd",
            "diff_sold": -0.7344431040000003,
        }
        self.assertEqual(tx, SolTransaction(**EXPECTED))

    # ========================================================================
    def testGetTransactionSimple2(self):
        tx = self.atlas_sol.getTransaction(
            signature="2u3AMnzckiFetYRjoZEy28H3z4JpFTfdFQxoGPdQQvKbySBtM8TwSbTZhfjzgg7jRhkVCBoKNJajXx7CxXkMjmPs"
        )
        EXPECTED = {
            "owner": "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1",
            "account": "4raiFCHdS9UixPnsPpbEnAHp94Fjp5qH9ZijuzHK1n9X",
            "mint": "GU4XtyVdkTKKJ6gyaBTHbNTTrXkpMxsNkziGW4RTpump",
            "wallet_prebalance": 91774486.983288,
            "wallet_postbalance": 100066425.275343,
            "diff": 8291938.292054996,
            "owner_cp": "EEdqNkpXofifBo8jGYCWdwu3yJHZZ3zLbiMBctLtQLCv",
            "account_cp": "B4HRBtqmx4QN5vqjNkJS246o31LkG7DcQ5c4VNxJSBG",
            "wallet_prebalance_cp": 8291938.292055,
            "wallet_postbalance_cp": 0,
            "mint_cp": "So11111111111111111111111111111111111111112",
            "wallet_prebalance_sold": 192.634345634,
            "wallet_postbalance_sold": 176.708434781,
            "account_sold": "6sENC2ee7HXBLGDtFpEwLXhkjYBAVu6ti86e6kfLxqya",
            "diff_sold": -15.925910853000005,
        }
        self.assertEqual(tx, SolTransaction(**EXPECTED))

    # ========================================================================
    def testGetTransfer(self):
        tx = self.atlas_sol.getTransaction(
            signature="5yBphEw19aAkNbsYLai2qModFVCHVae2W8ZKXt3sa9JCpJeGZKGHgA5AW7xK9QSBtDnHVWa19FYNVNtpWe1LgPXP"
        )
        EXPECTED = {
            "owner": "FnvLGtucz4E1ppJHRTev6Qv4X7g8Pw6WPStHCcbAKbfx",
            "account": "H3cnJE7YcistPDM11AWzR6CgLYHky9aYGLxcX5GU2ciJ",
            "mint": "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN",
            "wallet_prebalance": 200.000001,
            "wallet_postbalance": 0.0,
            "diff": -200.000001,
            "owner_cp": "DAnUT7fSUzGyUJwgpSE8pJEqtNGdGrLGA9GQP9C46vND",
            "account_cp": "ACoZ83R4hrB4z4h7RLd1RjVxdg57P3BjaAKr1xVE2Cuf",
            "wallet_prebalance_cp": 0,
            "wallet_postbalance_cp": 200.000001,
            "mint_cp": "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN",
            "wallet_prebalance_sold": 0,
            "wallet_postbalance_sold": 200.000001,
            "account_sold": "ACoZ83R4hrB4z4h7RLd1RjVxdg57P3BjaAKr1xVE2Cuf",
            "diff_sold": 200.000001,
        }
        self.assertEqual(tx, SolTransaction(**EXPECTED))


if __name__ == "__main__":
    unittest.main()
