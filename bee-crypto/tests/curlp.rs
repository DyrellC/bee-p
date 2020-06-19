// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

#[cfg(test)]
mod tests {
    use bee_crypto::ternary::{CurlP27, Sponge};
    use bee_ternary::{T1B1Buf, T3B1Buf, Trits, TryteBuf, T1B1};

    const INPUT_TRITS: &[i8] = &[
        -1, 1, -1, -1, 1, -1, 1, 1, 0, -1, 0, 0, 1, 0, 1, 0, 0, 0, -1, -1, -1, -1, 0, 0, -1, 0, 0, 1, 0, 0, -1, 0, 0,
        1, -1, -1, 1, -1, 1, -1, -1, 1, 0, 1, 0, 0, 0, 1, -1, 0, -1, 1, -1, -1, 0, 0, 0, -1, 0, 0, 1, -1, -1, 0, 0, 0,
        -1, 0, 0, 0, -1, -1, 0, 1, 1, -1, 1, 1, 1, 1, -1, 0, -1, 0, -1, 0, -1, 0, -1, -1, -1, -1, 0, 1, -1, 0, -1, -1,
        0, 0, 0, 0, 0, 1, 1, 0, 1, -1, 0, -1, -1, -1, 0, 0, 1, 0, -1, -1, -1, -1, 0, -1, -1, -1, 0, -1, 0, 0, -1, 1, 1,
        -1, -1, 1, 1, -1, 1, -1, 1, 0, -1, 1, -1, -1, -1, 0, 1, 1, 0, -1, 0, 1, 0, 0, 1, 1, 0, 0, -1, -1, 1, 0, 0, 0,
        0, -1, 1, 0, 1, 0, 0, 0, 1, -1, 1, -1, 0, 0, -1, 1, 1, -1, 0, 0, 1, -1, 0, 1, 0, -1, 1, -1, 0, 0, 1, -1, -1,
        -1, 0, 1, 0, -1, -1, 0, 1, 0, 0, 0, 1, -1, 1, -1, 0, 1, -1, -1, 0, 0, 0, -1, -1, 1, 1, 0, 1, -1, 0, 0, 0, -1,
        0, -1, 0, -1, -1, -1, -1, 0, 1, -1, -1, 0, 1,
    ];

    const EXPECTED_CURLP27_HASH_TRITS: &[i8] = &[
        -1, -1, -1, -1, 0, 0, 1, 1, -1, 1, 1, 0, -1, 1, 0, 1, 0, 0, 1, 0, -1, 1, 1, -1, -1, -1, 0, 1, 0, 1, -1, -1, 1,
        -1, -1, -1, -1, 1, 1, 1, 1, -1, 1, 1, 1, -1, 0, 1, -1, 1, 0, 0, 1, -1, 1, -1, 1, 0, 1, 0, 0, 1, -1, 1, 1, -1,
        0, 0, 1, 1, -1, 0, 1, 0, -1, 0, 0, 1, -1, -1, -1, 0, 0, -1, 1, 0, 0, -1, 1, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, -1,
        1, 0, -1, 1, 0, 1, 1, 0, 0, -1, 1, -1, 1, 0, -1, 0, 1, 0, 1, -1, 1, -1, 0, 1, 0, 1, 1, 1, -1, 0, 1, -1, 0, 0,
        0, 1, 0, -1, 0, -1, 0, -1, -1, 1, -1, 1, 1, 0, -1, 1, 0, -1, 1, 0, 1, -1, 0, 0, 0, -1, 0, 0, -1, 0, -1, -1, 0,
        0, -1, -1, 1, 1, -1, -1, -1, 0, -1, 0, -1, -1, 1, -1, -1, -1, -1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, -1, 1, 0, 1,
        -1, -1, -1, -1, 1, 0, 0, -1, 1, 1, 1, -1, 1, 0, -1, 0, 1, -1, 1, 1, 1, 0, 1, 1, 0, -1, 0, 1, 1, -1, 0, -1, 0,
        1, 0, 0, 1, 1, 1, -1, 0, 1, -1, 0,
    ];

    const INPUT_TRYTES: &str = "\
RSWWSFXPQJUBJROQBRQZWZXZJWMUBVIVMHPPTYSNW9YQIQQF9RCSJJCVZG9ZWITXNCSBBDHEEKDRBHVTWCZ9SZOOZHVB\
PCQNPKTWFNZAWGCZ9QDIMKRVINMIRZBPKRKQAIPGOHBTHTGYXTBJLSURDSPEOJ9UKJECUKCCPVIQQHDUYKVKISCEIEGV\
OQWRBAYXWGSJUTEVG9RPQLPTKYCRAJ9YNCUMDVDYDQCKRJOAPXCSUDAJGETALJINHEVNAARIPONBWXUOQUFGNOCUSSLY\
WKOZMZUKLNITZIFXFWQAYVJCVMDTRSHORGNSTKX9Z9DLWNHZSMNOYTU9AUCGYBVIITEPEKIXBCOFCMQPBGXYJKSHPXNU\
KFTXIJVYRFILAVXEWTUICZCYYPCEHNTK9SLGVL9RLAMYTAEPONCBHDXSEQZOXO9XCFUCPPMKEBR9IEJGQOPPILHFXHMI\
ULJYXZJASQEGCQDVYFOM9ETXAGVMSCHHQLFPATWOSMZIDL9AHMSDCE9UENACG9OVFAEIPPQYBCLXDMXXA9UBJFQQBCYK\
ETPNKHNOUKCSSYLWZDLKUARXNVKKKHNRBVSTVKQCZL9RY9BDTDTPUTFUBGRMSTOTXLWUHDMSGYRDSZLIPGQXIDMNCNBO\
AOI9WFUCXSRLJFIVTIPIAZUK9EDUJJ9B9YCJEZQQELLHVCWDNRH9FUXDGZRGOVXGOKORTCQQA9JXNROLETYCNLRMBGXB\
L9DQKMOAZCBJGWLNJLGRSTYBKLGFVRUF9QOPZVQFGMDJA9TBVGFJDBAHEVOLW9GNU9NICLCQJBOAJBAHHBZJGOFUCQMB\
GYQLCWNKSZPPBQMSJTJLM9GXOZHTNDLGIRCSIJAZTENQVQDHFSOQM9WVNWQQJNOPZMEISSCLOADMRNWALBBSLSWNCTOS\
NHNLWZBVCFIOGFPCPRKQSRGKFXGTWUSCPZSKQNLQJGKDLOXSBJMEHQPDZGSENUKWAHRNONDTBLHNAKGLOMCFYRCGMDOV\
ANPFHMQRFCZIQHCGVORJJNYMTORDKPJPLA9LWAKAWXLIFEVLKHRKCDG9QPQCPGVKIVBENQJTJGZKFTNZHIMQISVBNLHA\
YSSVJKTIELGTETKPVRQXNAPWOBGQGFRMMK9UQDWJHSQMYQQTCBMVQKUVGJEAGTEQDN9TCRRAZHDPSPIYVNKPGJSJZASZ\
QBM9WXEDWGAOQPPZFLAMZLEZGXPYSOJRWL9ZH9NOJTUKXNTCRRDO9GKULXBAVDRIZBOKJYVJUSHIX9F9O9ACYCAHUKBI\
EPVZWVJAJGSDQNZNWLIWVSKFJUMOYDMVUFLUXT9CEQEVRFBJVPCTJQCORM9JHLYFSMUVMFDXZFNCUFZZIKREIUIHUSHR\
PPOUKGFKWX9COXBAZMQBBFRFIBGEAVKBWKNTBMLPHLOUYOXPIQIZQWGOVUWQABTJT9ZZPNBABQFYRCQLXDHDEX9PULVT\
CQLWPTJLRSVZQEEYVBVY9KCNEZXQLEGADSTJBYOXEVGVTUFKNCNWMEDKDUMTKCMRPGKDCCBDHDVVSMPOPUBZOMZTXJSQ\
NVVGXNPPBVSBL9WWXWQNMHRMQFEQYKWNCSW9URI9FYPT9UZMAFMMGUKFYTWPCQKVJ9DIHRJFMXRZUGI9TMTFUQHGXNBI\
TDSORZORQIAMKY9VRYKLEHNRNFSEFBHF9KXIQAEZEJNQOENJVMWLMHI9GNZPXYUIFAJIVCLAGKUZIKTJKGNQVTXJORWI\
QDHUPBBPPYOUPFAABBVMMYATXERQHPECDVYGWDGXFJKOMOBXKRZD9MCQ9LGDGGGMYGUAFGMQTUHZOAPLKPNPCIKUNEMQ\
IZOCM9COAOMZSJ9GVWZBZYXMCNALENZ9PRYMHENPWGKX9ULUIGJUJRKFJPBTTHCRZQKEAHT9DC9GSWQEGDTZFHACZMLF\
YDVOWZADBNMEM9XXEOMHCNJMDSUAJRQTBUWKJF9RZHK9ACGUNI9URFIHLXBXCEODONPXBSCWP9WNAEYNALKQHGULUQGA\
FL9LB9NBLLCACLQFGQMXRHGBTMI9YKAJKVELRWWKJAPKMSYMJTDYMZ9PJEEYIRXRMMFLRSFSHIXUL9NEJABLRUGHJFL9\
RASMSKOI9VCFRZ9GWTMODUUESIJBHWWHZYCLDENBFSJQPIOYC9MBGOOXSWEMLVU9L9WJXKZKVDBDMFSVHHISSSNILUMW\
ULMVMESQUIHDGBDXROXGH9MTNFSLWJZRAPOKKRGXAAQBFPYPAAXLSTMNSNDTTJQSDQORNJS9BBGQ9KQJZYPAQ9JYQZJ9\
B9KQDAXUACZWRUNGMBOQLQZUHFNCKVQGORRZGAHES9PWJUKZWUJSBMNZFILBNBQQKLXITCTQDDBV9UDAOQOUPWMXTXWF\
WVMCXIXLRMRWMAYYQJPCEAAOFEOGZQMEDAGYGCTKUJBS9AGEXJAFHWWDZRYEN9DN9HVCMLFURISLYSWKXHJKXMHUWZXU\
QARMYPGKRKQMHVR9JEYXJRPNZINYNCGZHHUNHBAIJHLYZIZGGIDFWVNXZQADLEDJFTIUTQWCQSX9QNGUZXGXJYUUTFSZ\
PQKXBA9DFRQRLTLUJENKESDGTZRGRSLTNYTITXRXRGVLWBTEWPJXZYLGHLQBAVYVOSABIVTQYQM9FIQKCBRRUEMVVTME\
RLWOK\
";

    const EXPECTED_CURLP27_HASH_TRYTES: &str = "\
KXRVLFETGUTUWBCNCC9DWO99JQTEI9YXVOZHWELSYP9SG9KN9WCKXOVTEFHFH9EFZJKFYCZKQPPBXYSGJ\
";

    #[test]
    fn verify_curlp27_hash_trytes() {
        let mut curlp27 = CurlP27::new();

        let input_trytes = TryteBuf::try_from_str(INPUT_TRYTES);
        assert!(input_trytes.is_ok());
        let input_trytes = input_trytes.unwrap();

        let input_trit_buf = input_trytes.as_trits().encode::<T1B1Buf>();

        let expected_hash = TryteBuf::try_from_str(EXPECTED_CURLP27_HASH_TRYTES);
        assert!(expected_hash.is_ok());
        let expected_hash = expected_hash.unwrap();

        let calculated_hash = curlp27.digest(&input_trit_buf);
        assert!(
            calculated_hash.is_ok(),
            "<CurlP27 as Sponge>::Error is Infallible and this assert should never fail"
        );
        let calculated_hash = calculated_hash.unwrap().encode::<T3B1Buf>();

        assert_eq!(calculated_hash.as_slice(), expected_hash.as_trits());
    }

    #[test]
    fn verify_curlp27_hash_trits() {
        let mut curlp27 = CurlP27::new();

        let input_trits = unsafe { Trits::<T1B1>::from_raw_unchecked(INPUT_TRITS, INPUT_TRITS.len()) };
        let expected_hash = unsafe {
            Trits::<T1B1>::from_raw_unchecked(EXPECTED_CURLP27_HASH_TRITS, EXPECTED_CURLP27_HASH_TRITS.len())
        };

        let calculated_hash = curlp27.digest(&input_trits);
        assert!(
            calculated_hash.is_ok(),
            "<CurlP27 as Sponge>::Error is Infallible and this assert should never fail"
        );
        assert_eq!(expected_hash, &*calculated_hash.unwrap());
    }
}
