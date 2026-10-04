use crate::FrameBuffer;
use serde::{Deserialize, Serialize};

pub const BENCHMARK_SUITE_V1_ID: &str = "phicade-agent-gym-suite-v1";
pub const BENCHMARK_SUITE_V2_ID: &str = "phicade-agent-gym-suite-v2";
pub const BENCHMARK_SUITE_V3_ID: &str = "phicade-agent-gym-suite-v3";
pub const BENCHMARK_SUITE_V4_ID: &str = "phicade-agent-gym-suite-v4";
pub const BENCHMARK_SUITE_V5_ID: &str = "phicade-agent-gym-suite-v5";
pub const BENCHMARK_SUITE_V6_ID: &str = "phicade-agent-gym-suite-v6";
pub const BENCHMARK_SUITE_V7_ID: &str = "phicade-agent-gym-suite-v7";
pub const BENCHMARK_SUITE_V8_ID: &str = "phicade-agent-gym-suite-v8";
pub const BENCHMARK_SUITE_V9_ID: &str = "phicade-agent-gym-suite-v9";
pub const BENCHMARK_SUITE_V10_ID: &str = "phicade-agent-gym-suite-v10";
pub const BENCHMARK_SUITE_V11_ID: &str = "phicade-agent-gym-suite-v11";

pub const AGENT_GYM_ID: &str = "move-block-to-x-v1";
pub const AGENT_GYM_ROM_SHA256: &str =
    "353e69e859f50f5ef14f0221e386b18b8194f771cc603696530a59617593c59e";
pub const AGENT_GYM_SOURCE_SHA256: &str =
    "0c82f65532030d64bf022539002f334716a5aae822672b3cdfca511906e51e6f";

pub const AGENT_GYM_MIRROR_ID: &str = "move-block-to-x-mirror-v1";
pub const AGENT_GYM_MIRROR_ROM_SHA256: &str =
    "278a8106343fe1688a1370c0575578417744c96ae52568ab1e97f446dc222bfb";
pub const AGENT_GYM_MIRROR_SOURCE_SHA256: &str =
    "fc10866cf7166f74f1ae41f8957f3b6057d8bf710731e42a02cbdb9087feb658";

pub const AGENT_GYM_WALL_ID: &str = "wall-detour-v1";
pub const AGENT_GYM_WALL_ROM_SHA256: &str =
    "c8bfbe95b368635370a61abf004d0efd4135b96c8fa9e4fb510582e25534c4f8";
pub const AGENT_GYM_WALL_SOURCE_SHA256: &str =
    "02fc444e7ffc6f9de819f7462685448268b41a641592332918f972e17fd0fc96";

pub const AGENT_GYM_TEMPORAL_LEFT_ID: &str = "temporal-cue-left-v1";
pub const AGENT_GYM_TEMPORAL_LEFT_ROM_SHA256: &str =
    "da16bdda571bd2f3d5581097e1643ce7496ae3ea586f3b1756330a2b3fc171f5";
pub const AGENT_GYM_TEMPORAL_LEFT_SOURCE_SHA256: &str =
    "450ecadadb3ed51e1e613e4edd06007880309c4841fbae313bb45f0a75932a4c";

pub const AGENT_GYM_TEMPORAL_RIGHT_ID: &str = "temporal-cue-right-v1";
pub const AGENT_GYM_TEMPORAL_RIGHT_ROM_SHA256: &str =
    "eefb364a47b68267138d35b402d2d6f5de1d70940318cac947fb400d6e210337";
pub const AGENT_GYM_TEMPORAL_RIGHT_SOURCE_SHA256: &str =
    "8c814246cb6948d50d5242ef2e01369c90a5965cb2151da5b9e4f27fff5e282f";

pub const AGENT_GYM_RELAY_LEFT_ID: &str = "relay-rooms-left-v1";
pub const AGENT_GYM_RELAY_LEFT_ROM_SHA256: &str =
    "563a0b902f9e93d8887be830302e938f5ad394bf30c052f926511765486dae7e";
pub const AGENT_GYM_RELAY_LEFT_SOURCE_SHA256: &str =
    "75011896544ab2a2c1285d5fa667af22fbc329334e7d96601642577cf4d73bac";

pub const AGENT_GYM_RELAY_RIGHT_ID: &str = "relay-rooms-right-v1";
pub const AGENT_GYM_RELAY_RIGHT_ROM_SHA256: &str =
    "83beb9b0c08754aaefa2f4d2598cba8548d4e56d7139487ae7fb84c3dd75f242";
pub const AGENT_GYM_RELAY_RIGHT_SOURCE_SHA256: &str =
    "0408d7f1d8325e9e12564a33aa45b8814e889c689e53116a77b13c685657b739";

pub const AGENT_GYM_KEY_GATE_LEFT_ID: &str = "key-gate-left-v1";
pub const AGENT_GYM_KEY_GATE_LEFT_ROM_SHA256: &str =
    "904492aa3be9ebfca1f02ff220417eb94ee3f12332dfa4e2263e20c4009094db";
pub const AGENT_GYM_KEY_GATE_LEFT_SOURCE_SHA256: &str =
    "d9fcf741c4c5b78faa7276b61829321950384ada58a69f5ab07ed53fe982fa5c";

pub const AGENT_GYM_KEY_GATE_RIGHT_ID: &str = "key-gate-right-v1";
pub const AGENT_GYM_KEY_GATE_RIGHT_ROM_SHA256: &str =
    "72553ffab515b83e9548b454e85491246b6260474bdddb8d3011c0be1b631442";
pub const AGENT_GYM_KEY_GATE_RIGHT_SOURCE_SHA256: &str =
    "e22c6d8f9b42105a5c6d4cb706631736c9cc61738ea8e94a8162c7761435961c";

pub const AGENT_GYM_POWER_CHAIN_LEFT_ID: &str = "power-chain-left-v1";
pub const AGENT_GYM_POWER_CHAIN_LEFT_ROM_SHA256: &str =
    "fd84eb10474a0834817101b99a5efc8b233d9b13a4c8d6d784c9394320636e40";
pub const AGENT_GYM_POWER_CHAIN_LEFT_SOURCE_SHA256: &str =
    "59a619dc42500e8ee1488474869252df41533d58159da247e0df2793222b4afe";

pub const AGENT_GYM_POWER_CHAIN_RIGHT_ID: &str = "power-chain-right-v1";
pub const AGENT_GYM_POWER_CHAIN_RIGHT_ROM_SHA256: &str =
    "33026212377de8c08dec00dbc7d6bc4852902c093d87624916bfcc3bc69e9b9f";
pub const AGENT_GYM_POWER_CHAIN_RIGHT_SOURCE_SHA256: &str =
    "687f150dbf82bb634c8975362d02b967f1aa26299ac36380bbfa0eb99d4e68eb";

pub const AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_ID: &str = "branch-selector-triangle-v1";
pub const AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_ROM_SHA256: &str =
    "80c1dbd415f14a16823a7903b7117564b352ee2fe7d151d9c41c9f51fef20505";
pub const AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_SOURCE_SHA256: &str =
    "b58ad96706d25724f85d6802afc499353419aeb09bd0f421b02e764f08d6e071";

pub const AGENT_GYM_BRANCH_SELECTOR_SQUARE_ID: &str = "branch-selector-square-v1";
pub const AGENT_GYM_BRANCH_SELECTOR_SQUARE_ROM_SHA256: &str =
    "300d827be392f9859fe85e0b0eb06a0af329b3148ba222295ee47dcd5290e10a";
pub const AGENT_GYM_BRANCH_SELECTOR_SQUARE_SOURCE_SHA256: &str =
    "0dc8ba01a974cc467349058cd92e0a50e2168a67bcbff3dea36f64ddfc376a35";

pub const AGENT_GYM_NESTED_TRIANGLE_CIRCLE_ID: &str = "nested-branch-triangle-circle-v1";
pub const AGENT_GYM_NESTED_TRIANGLE_CIRCLE_ROM_SHA256: &str =
    "85bebe2dca1974b87c434abde755ec138a6f398f2e33b9cfcf81fae1db4914f5";
pub const AGENT_GYM_NESTED_TRIANGLE_CIRCLE_SOURCE_SHA256: &str =
    "2bb724ae134c2e7ec31c24b3096ac08e3f94158f24e75c50ff37b871fdce7a82";

pub const AGENT_GYM_NESTED_TRIANGLE_CROSS_ID: &str = "nested-branch-triangle-cross-v1";
pub const AGENT_GYM_NESTED_TRIANGLE_CROSS_ROM_SHA256: &str =
    "5e96ec33e6b5a33776bebef68a63a311087cd51bca5976a60ef3aad55ca3e5b2";
pub const AGENT_GYM_NESTED_TRIANGLE_CROSS_SOURCE_SHA256: &str =
    "d65fbb5339300243d33de0d1544eb1abc8c87ec626fa126b33b56f91657dcde6";

pub const AGENT_GYM_NESTED_SQUARE_CIRCLE_ID: &str = "nested-branch-square-circle-v1";
pub const AGENT_GYM_NESTED_SQUARE_CIRCLE_ROM_SHA256: &str =
    "7779a57acf881c39709e1b31f561a361ab6539f1aed36ab875cacab094726f0f";
pub const AGENT_GYM_NESTED_SQUARE_CIRCLE_SOURCE_SHA256: &str =
    "051ffafd163aa65c310145c999c76bedbf90978e74503901a6ed82d1aa8a7137";

pub const AGENT_GYM_NESTED_SQUARE_CROSS_ID: &str = "nested-branch-square-cross-v1";
pub const AGENT_GYM_NESTED_SQUARE_CROSS_ROM_SHA256: &str =
    "9ca574ca0c55540ccc734146f8230c2f26bb6b1b98f87c43f3b186f9d57e80d6";
pub const AGENT_GYM_NESTED_SQUARE_CROSS_SOURCE_SHA256: &str =
    "eb915d89c6241f71150854d32c1f768079865a895e8f62137c8bdf76409994fd";

pub const AGENT_GYM_BINDING_NORMAL_TRIANGLE_ID: &str = "binding-memory-normal-triangle-v1";
pub const AGENT_GYM_BINDING_NORMAL_TRIANGLE_ROM_SHA256: &str =
    "5f96edb28dbc65ae6f37d2f0bc740ef2ce9e2c2b8b9191a8e1a9882eaffd1a95";
pub const AGENT_GYM_BINDING_NORMAL_TRIANGLE_SOURCE_SHA256: &str =
    "1a0b386a2cfa4a8e6caf93e58e40620dd8913f068cf6a90bb09159cafbf506da";

pub const AGENT_GYM_BINDING_NORMAL_SQUARE_ID: &str = "binding-memory-normal-square-v1";
pub const AGENT_GYM_BINDING_NORMAL_SQUARE_ROM_SHA256: &str =
    "ee5eeac96f5073605014ad2e709331d96c7771a0b076f332f38ec5ec6356270d";
pub const AGENT_GYM_BINDING_NORMAL_SQUARE_SOURCE_SHA256: &str =
    "d2607daadac805fe27901ff1840639293f5cadb06b40e86ff5fdb477cd148faa";

pub const AGENT_GYM_BINDING_SWAPPED_TRIANGLE_ID: &str = "binding-memory-swapped-triangle-v1";
pub const AGENT_GYM_BINDING_SWAPPED_TRIANGLE_ROM_SHA256: &str =
    "31cfb105608c2e33c0cf1f43be7c8c037f0a56a4401aa4abfd43ecc6ff694300";
pub const AGENT_GYM_BINDING_SWAPPED_TRIANGLE_SOURCE_SHA256: &str =
    "41f8b37237832f824275f307beacabbd3731e86c507ac3098ec206b4b4cdbf2e";

pub const AGENT_GYM_BINDING_SWAPPED_SQUARE_ID: &str = "binding-memory-swapped-square-v1";
pub const AGENT_GYM_BINDING_SWAPPED_SQUARE_ROM_SHA256: &str =
    "c6f9a0d73187dbec822753c026405199c6d278dee19b4343de524e6fd4568492";
pub const AGENT_GYM_BINDING_SWAPPED_SQUARE_SOURCE_SHA256: &str =
    "e1773c96b87e92f51e9129ed54f6cb60ca3cd4d03f35d1ed287edba915460c4d";

pub const AGENT_GYM_COMP_NTM_ID: &str = "compositional-recall-normal-triangle-match-v1";
pub const AGENT_GYM_COMP_NTM_ROM_SHA256: &str =
    "a0dd1c4fd34a308c9951fee576f687a9337bef9c1dd05d38e751ef48e7949720";
pub const AGENT_GYM_COMP_NTM_SOURCE_SHA256: &str =
    "d9c6f0e0462502a2e5906a9a8f07861526bdc65d2f99758db57b630e6233d6e7";

pub const AGENT_GYM_COMP_NTF_ID: &str = "compositional-recall-normal-triangle-flip-v1";
pub const AGENT_GYM_COMP_NTF_ROM_SHA256: &str =
    "3dcf0413ad3d62a49d45167a108a7483755925259ec8527ef2e9772d07b4f8d0";
pub const AGENT_GYM_COMP_NTF_SOURCE_SHA256: &str =
    "8b555998f193fd5cb40e61358da986119698e522938e213670c6e1d3c6a3dde4";

pub const AGENT_GYM_COMP_NSM_ID: &str = "compositional-recall-normal-square-match-v1";
pub const AGENT_GYM_COMP_NSM_ROM_SHA256: &str =
    "8aa58e9e982594462cce0b48aab2a9b6a060537e1a404b185511f9b6598f4afe";
pub const AGENT_GYM_COMP_NSM_SOURCE_SHA256: &str =
    "58659c6891709c5524644f40b0626ec0fca1df7ba5c36fada31f6a0611374eee";

pub const AGENT_GYM_COMP_NSF_ID: &str = "compositional-recall-normal-square-flip-v1";
pub const AGENT_GYM_COMP_NSF_ROM_SHA256: &str =
    "aa10327cd7ef0d06b8506d062b72bcfe6d4199759def0f31fdd6e44da6beb83e";
pub const AGENT_GYM_COMP_NSF_SOURCE_SHA256: &str =
    "f77ebc811b75f15d47845246e702650d4f27129f5329eb767bd1c19b7d3c0123";

pub const AGENT_GYM_COMP_STM_ID: &str = "compositional-recall-swapped-triangle-match-v1";
pub const AGENT_GYM_COMP_STM_ROM_SHA256: &str =
    "7732b4fc7220698bb20e3a40fd6c8bb85fdfa608bcc75f9135c6743e2453da57";
pub const AGENT_GYM_COMP_STM_SOURCE_SHA256: &str =
    "b145f5b94fd0af906020257c07d8f4167c70b6e95fbc37677e92f427e1dc98d2";

pub const AGENT_GYM_COMP_STF_ID: &str = "compositional-recall-swapped-triangle-flip-v1";
pub const AGENT_GYM_COMP_STF_ROM_SHA256: &str =
    "5717f04ab690b1a887448205358f17aee64a590e3a0011b7fec039183ff6bea4";
pub const AGENT_GYM_COMP_STF_SOURCE_SHA256: &str =
    "2bab8e7b334d0fbeeb6c1f3901fe51aec2fbddeb450ed6d710853e6058362108";

pub const AGENT_GYM_COMP_SSM_ID: &str = "compositional-recall-swapped-square-match-v1";
pub const AGENT_GYM_COMP_SSM_ROM_SHA256: &str =
    "1174a6b2b158b3712a9a71b880983259bfe4a8e26f166193f56e86e5b09886dd";
pub const AGENT_GYM_COMP_SSM_SOURCE_SHA256: &str =
    "886a3dab4a3814b8a9723e94353daff511f1c510976e7f3766a84dbda8ac75b9";

pub const AGENT_GYM_COMP_SSF_ID: &str = "compositional-recall-swapped-square-flip-v1";
pub const AGENT_GYM_COMP_SSF_ROM_SHA256: &str =
    "c86b4bbacc6e9bedc9d731d2b396d5bef64889ecca2243031289d7f1020a996b";
pub const AGENT_GYM_COMP_SSF_SOURCE_SHA256: &str =
    "1e1f8380981896ddca16fa5f89ab7915175e94476e2c3cf2fdd425fb895f2613";

pub const AGENT_GYM_SEQ_NTMM_ID: &str = "sequential-rule-normal-triangle-match-match-v1";
pub const AGENT_GYM_SEQ_NTMM_ROM_SHA256: &str =
    "0cee85d2c39a9c37ac530a7c1b73c0de9afb43040eb94153efa4edbcefe778cc";
pub const AGENT_GYM_SEQ_NTMM_SOURCE_SHA256: &str =
    "79a5fcd7a3c503b00dbff2b59c1eb31f45f42c4c24fde0bf66ec26090de4ddc9";

pub const AGENT_GYM_SEQ_NTMF_ID: &str = "sequential-rule-normal-triangle-match-flip-v1";
pub const AGENT_GYM_SEQ_NTMF_ROM_SHA256: &str =
    "8d382853a99a38b967a397cb166733c49fa3df87d9364c5bbb0d6816d77dbe13";
pub const AGENT_GYM_SEQ_NTMF_SOURCE_SHA256: &str =
    "4c13efc70adef3322a243dc41bb7d6c185f00282d02917a3712a574366f0051f";

pub const AGENT_GYM_SEQ_NTFM_ID: &str = "sequential-rule-normal-triangle-flip-match-v1";
pub const AGENT_GYM_SEQ_NTFM_ROM_SHA256: &str =
    "3f9055e1feefda8325c08dddcb311955de013f237dcc55e49603eefa21d8eca9";
pub const AGENT_GYM_SEQ_NTFM_SOURCE_SHA256: &str =
    "c25a6b5158629209a4fd524e88f9af0d2b1681216b8e0237cbba8604519f01ad";

pub const AGENT_GYM_SEQ_NTFF_ID: &str = "sequential-rule-normal-triangle-flip-flip-v1";
pub const AGENT_GYM_SEQ_NTFF_ROM_SHA256: &str =
    "8f1ef5abac3100f688b1fa13adee4ff89db74b7906f30f27d9829d2dfd2e1b8e";
pub const AGENT_GYM_SEQ_NTFF_SOURCE_SHA256: &str =
    "b0b7ad42bbdba665ee74c233ee0895f69fc73da3e3d6b04e10218881ec815d1d";

pub const AGENT_GYM_SEQ_NSMM_ID: &str = "sequential-rule-normal-square-match-match-v1";
pub const AGENT_GYM_SEQ_NSMM_ROM_SHA256: &str =
    "460ddc6e00b5b29384a784acd162d6b1142348bfbe81544d784c16e57c4e3623";
pub const AGENT_GYM_SEQ_NSMM_SOURCE_SHA256: &str =
    "e0d8739aa1dbdd853c42c5c8fa09ffbec6e7a4bd7bb0958759ace04570f9b58d";

pub const AGENT_GYM_SEQ_NSMF_ID: &str = "sequential-rule-normal-square-match-flip-v1";
pub const AGENT_GYM_SEQ_NSMF_ROM_SHA256: &str =
    "b91d69651b6b6644f5d4b5bb9b186145e3ae442731bdb88197a8263b8785839f";
pub const AGENT_GYM_SEQ_NSMF_SOURCE_SHA256: &str =
    "09ef1f71a27b95413175b5fc36b1672042ec6985800804beb53df3e42ea2fef1";

pub const AGENT_GYM_SEQ_NSFM_ID: &str = "sequential-rule-normal-square-flip-match-v1";
pub const AGENT_GYM_SEQ_NSFM_ROM_SHA256: &str =
    "fce4ccce7c7ad284ba5abab7dead5e59efaad21135878301a63576fe19d47422";
pub const AGENT_GYM_SEQ_NSFM_SOURCE_SHA256: &str =
    "ae45b59057ffc4a94de22f76acf8e2f8fd8a3158966a07e77add53ee812092d4";

pub const AGENT_GYM_SEQ_NSFF_ID: &str = "sequential-rule-normal-square-flip-flip-v1";
pub const AGENT_GYM_SEQ_NSFF_ROM_SHA256: &str =
    "20a2946cce09f78c4a9c74b01ce53ae13479775faed017479b18227bf2be4da6";
pub const AGENT_GYM_SEQ_NSFF_SOURCE_SHA256: &str =
    "7dacaff82f558d91407b876233847af2e729d37485660096db49792f53ed63e2";

pub const AGENT_GYM_SEQ_STMM_ID: &str = "sequential-rule-swapped-triangle-match-match-v1";
pub const AGENT_GYM_SEQ_STMM_ROM_SHA256: &str =
    "2b49b2a09d73d8c3670ffb3a16fc1e91f5360c24c2f8efaeb9e842dfbf46a739";
pub const AGENT_GYM_SEQ_STMM_SOURCE_SHA256: &str =
    "7d43e44c60f88b53657efb2a98cf96fe839ada5da267c398014ab88adc115882";

pub const AGENT_GYM_SEQ_STMF_ID: &str = "sequential-rule-swapped-triangle-match-flip-v1";
pub const AGENT_GYM_SEQ_STMF_ROM_SHA256: &str =
    "d3298c31db90ec1e920b2fb085d8ed686759266bc8625348c15850bff6795780";
pub const AGENT_GYM_SEQ_STMF_SOURCE_SHA256: &str =
    "eef19eea18ea136862c9c1184e9a5e3da884ed7314a5e4122ca6e9f48c4ff484";

pub const AGENT_GYM_SEQ_STFM_ID: &str = "sequential-rule-swapped-triangle-flip-match-v1";
pub const AGENT_GYM_SEQ_STFM_ROM_SHA256: &str =
    "8ae8396bf1deeb7c686b8bcf75a6351f55693942ee91bb18465f0e95606d7cbc";
pub const AGENT_GYM_SEQ_STFM_SOURCE_SHA256: &str =
    "30c7e6788393596b9cafe0b62e7c080c775d0d76c6238393f333ff636cbb3edc";

pub const AGENT_GYM_SEQ_STFF_ID: &str = "sequential-rule-swapped-triangle-flip-flip-v1";
pub const AGENT_GYM_SEQ_STFF_ROM_SHA256: &str =
    "165160108ffd735a0aab52d1252d61520969821261ec16a839391e92b1ea3aed";
pub const AGENT_GYM_SEQ_STFF_SOURCE_SHA256: &str =
    "6c0496ed6107124cb39bc6c96293ba50971bbfac7084bcfc7f4dd5cf5561599c";

pub const AGENT_GYM_SEQ_SSMM_ID: &str = "sequential-rule-swapped-square-match-match-v1";
pub const AGENT_GYM_SEQ_SSMM_ROM_SHA256: &str =
    "91a505ff8c2c1c17e4796c878873636c4ceaa36aa19f57d9c9d37a44a60ff61f";
pub const AGENT_GYM_SEQ_SSMM_SOURCE_SHA256: &str =
    "e58497c3410e8c16711b05c48725c4e866e0953d326f070846a6e11acaaee733";

pub const AGENT_GYM_SEQ_SSMF_ID: &str = "sequential-rule-swapped-square-match-flip-v1";
pub const AGENT_GYM_SEQ_SSMF_ROM_SHA256: &str =
    "fe2546aba524ff82f89a03b7b67e162108ee58863c1ba960c427c1d03ddf6929";
pub const AGENT_GYM_SEQ_SSMF_SOURCE_SHA256: &str =
    "468d5e1a2b7a7f4f2fc248ad123e88ae4d966a24032d610403fcf9dc695a3c91";

pub const AGENT_GYM_SEQ_SSFM_ID: &str = "sequential-rule-swapped-square-flip-match-v1";
pub const AGENT_GYM_SEQ_SSFM_ROM_SHA256: &str =
    "e94ef007dd822ec1c6921df6e534b724d0e5268ca9fcaf06d7ed7e161d677068";
pub const AGENT_GYM_SEQ_SSFM_SOURCE_SHA256: &str =
    "6145ada1bdb22ac3e3e3fead6893e08cd31df4edab0d62cf5dd1b3b667fb061b";

pub const AGENT_GYM_SEQ_SSFF_ID: &str = "sequential-rule-swapped-square-flip-flip-v1";
pub const AGENT_GYM_SEQ_SSFF_ROM_SHA256: &str =
    "c707db53b771b2a13a0cade7975db0d95ff13b4e8e065d0da3af16f4a0894014";
pub const AGENT_GYM_SEQ_SSFF_SOURCE_SHA256: &str =
    "eedf017f31d079a2e796b8c12e95d33bb2c3ea0233b95dc7cc5abb1fd3a1cd55";

pub const AGENT_GYM_TARGET_X: i32 = 136;
pub const AGENT_GYM_TARGET_Y: i32 = 112;
pub const AGENT_GYM_START_X: i32 = 16;
pub const AGENT_GYM_START_Y: i32 = 24;
pub const AGENT_GYM_INITIAL_DISTANCE: i32 = 208;
pub const AGENT_GYM_SUCCESS_DISTANCE: i32 = 4;
pub const AGENT_GYM_WARMUP_FRAMES: u64 = 120;
pub const AGENT_GYM_PLAYER_SIZE: usize = 8;

pub const AGENT_GYM_MIRROR_START_X: i32 = 136;
pub const AGENT_GYM_MIRROR_START_Y: i32 = 112;
pub const AGENT_GYM_MIRROR_TARGET_X: i32 = 16;
pub const AGENT_GYM_MIRROR_TARGET_Y: i32 = 24;

pub const AGENT_GYM_WALL_START_X: i32 = 16;
pub const AGENT_GYM_WALL_START_Y: i32 = 24;
pub const AGENT_GYM_WALL_TARGET_X: i32 = 136;
pub const AGENT_GYM_WALL_TARGET_Y: i32 = 24;
pub const AGENT_GYM_WALL_INITIAL_DISTANCE: i32 = 120;

pub const AGENT_GYM_TEMPORAL_START_X: i32 = 72;
pub const AGENT_GYM_TEMPORAL_START_Y: i32 = 96;
pub const AGENT_GYM_TEMPORAL_LEFT_TARGET_X: i32 = 24;
pub const AGENT_GYM_TEMPORAL_RIGHT_TARGET_X: i32 = 120;
pub const AGENT_GYM_TEMPORAL_TARGET_Y: i32 = 96;
pub const AGENT_GYM_TEMPORAL_INITIAL_DISTANCE: i32 = 48;

pub const AGENT_GYM_RELAY_START_X: i32 = 72;
pub const AGENT_GYM_RELAY_START_Y: i32 = 24;
pub const AGENT_GYM_RELAY_LEFT_TARGET_X: i32 = 24;
pub const AGENT_GYM_RELAY_RIGHT_TARGET_X: i32 = 120;
pub const AGENT_GYM_RELAY_TARGET_Y: i32 = 112;
pub const AGENT_GYM_RELAY_INITIAL_DISTANCE: i32 = 136;

pub const AGENT_GYM_KEY_GATE_START_X: i32 = 72;
pub const AGENT_GYM_KEY_GATE_START_Y: i32 = 112;
pub const AGENT_GYM_KEY_GATE_TARGET_X: i32 = 72;
pub const AGENT_GYM_KEY_GATE_TARGET_Y: i32 = 24;
pub const AGENT_GYM_KEY_GATE_INITIAL_DISTANCE: i32 = 88;

pub const AGENT_GYM_POWER_CHAIN_START_X: i32 = 72;
pub const AGENT_GYM_POWER_CHAIN_START_Y: i32 = 112;
pub const AGENT_GYM_POWER_CHAIN_TARGET_X: i32 = 72;
pub const AGENT_GYM_POWER_CHAIN_TARGET_Y: i32 = 24;
pub const AGENT_GYM_POWER_CHAIN_INITIAL_DISTANCE: i32 = 88;

pub const AGENT_GYM_BRANCH_SELECTOR_START_X: i32 = 72;
pub const AGENT_GYM_BRANCH_SELECTOR_START_Y: i32 = 112;
pub const AGENT_GYM_BRANCH_SELECTOR_TARGET_X: i32 = 72;
pub const AGENT_GYM_BRANCH_SELECTOR_TARGET_Y: i32 = 24;
pub const AGENT_GYM_BRANCH_SELECTOR_INITIAL_DISTANCE: i32 = 88;

pub const AGENT_GYM_NESTED_START_X: i32 = 72;
pub const AGENT_GYM_NESTED_START_Y: i32 = 112;
pub const AGENT_GYM_NESTED_TARGET_X: i32 = 72;
pub const AGENT_GYM_NESTED_TARGET_Y: i32 = 24;
pub const AGENT_GYM_NESTED_INITIAL_DISTANCE: i32 = 88;

pub const AGENT_GYM_BINDING_START_X: i32 = 72;
pub const AGENT_GYM_BINDING_START_Y: i32 = 96;
pub const AGENT_GYM_BINDING_LEFT_TARGET_X: i32 = 24;
pub const AGENT_GYM_BINDING_RIGHT_TARGET_X: i32 = 120;
pub const AGENT_GYM_BINDING_TARGET_Y: i32 = 96;
pub const AGENT_GYM_BINDING_INITIAL_DISTANCE: i32 = 48;

pub static AGENT_GYM_DPAD_BUTTONS: [&str; 4] = ["UP", "DOWN", "LEFT", "RIGHT"];
pub static AGENT_GYM_TEMPORAL_BUTTONS: [&str; 3] = ["A", "LEFT", "RIGHT"];
pub static AGENT_GYM_RELAY_BUTTONS: [&str; 5] = ["A", "UP", "DOWN", "LEFT", "RIGHT"];
pub static AGENT_GYM_KEY_GATE_BUTTONS: [&str; 5] = ["A", "UP", "DOWN", "LEFT", "RIGHT"];
pub static AGENT_GYM_POWER_CHAIN_BUTTONS: [&str; 5] = ["A", "UP", "DOWN", "LEFT", "RIGHT"];
pub static AGENT_GYM_BRANCH_SELECTOR_BUTTONS: [&str; 5] = ["A", "UP", "DOWN", "LEFT", "RIGHT"];
pub static AGENT_GYM_NESTED_BUTTONS: [&str; 5] = ["A", "UP", "DOWN", "LEFT", "RIGHT"];
pub static AGENT_GYM_BINDING_BUTTONS: [&str; 3] = ["A", "LEFT", "RIGHT"];
pub static AGENT_GYM_COMPOSITIONAL_BUTTONS: [&str; 3] = ["A", "LEFT", "RIGHT"];
pub static AGENT_GYM_SEQUENTIAL_BUTTONS: [&str; 3] = ["A", "LEFT", "RIGHT"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PixelPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OracleLeg {
    pub button: &'static str,
    pub frames: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkTaskSpec {
    /// Suite where this task was first introduced. Suite membership is owned by
    /// BenchmarkSuiteSpec so one frozen task may belong to multiple suite versions.
    pub suite_id: &'static str,
    pub id: &'static str,
    pub title: &'static str,
    pub rom_sha256: &'static str,
    pub source_sha256: &'static str,
    pub start: PixelPoint,
    pub target: PixelPoint,
    pub initial_distance: i32,
    pub success_distance: i32,
    pub warmup_frames: u64,
    pub allowed_buttons: &'static [&'static str],
    pub prompt: &'static str,
    pub oracle: &'static [OracleLeg],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkSuiteSpec {
    pub id: &'static str,
    pub title: &'static str,
    pub version: u16,
    pub tasks: &'static [BenchmarkTaskSpec],
}

pub const AGENT_GYM_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_TARGET_X,
    y: AGENT_GYM_TARGET_Y,
};

pub const AGENT_GYM_START: PixelPoint = PixelPoint {
    x: AGENT_GYM_START_X,
    y: AGENT_GYM_START_Y,
};

pub const AGENT_GYM_MIRROR_START: PixelPoint = PixelPoint {
    x: AGENT_GYM_MIRROR_START_X,
    y: AGENT_GYM_MIRROR_START_Y,
};

pub const AGENT_GYM_MIRROR_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_MIRROR_TARGET_X,
    y: AGENT_GYM_MIRROR_TARGET_Y,
};

pub const AGENT_GYM_WALL_START: PixelPoint = PixelPoint {
    x: AGENT_GYM_WALL_START_X,
    y: AGENT_GYM_WALL_START_Y,
};

pub const AGENT_GYM_WALL_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_WALL_TARGET_X,
    y: AGENT_GYM_WALL_TARGET_Y,
};

pub const AGENT_GYM_TEMPORAL_START: PixelPoint = PixelPoint {
    x: AGENT_GYM_TEMPORAL_START_X,
    y: AGENT_GYM_TEMPORAL_START_Y,
};

pub const AGENT_GYM_TEMPORAL_LEFT_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_TEMPORAL_LEFT_TARGET_X,
    y: AGENT_GYM_TEMPORAL_TARGET_Y,
};

pub const AGENT_GYM_TEMPORAL_RIGHT_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_TEMPORAL_RIGHT_TARGET_X,
    y: AGENT_GYM_TEMPORAL_TARGET_Y,
};

pub const AGENT_GYM_RELAY_START: PixelPoint = PixelPoint {
    x: AGENT_GYM_RELAY_START_X,
    y: AGENT_GYM_RELAY_START_Y,
};

pub const AGENT_GYM_RELAY_LEFT_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_RELAY_LEFT_TARGET_X,
    y: AGENT_GYM_RELAY_TARGET_Y,
};

pub const AGENT_GYM_RELAY_RIGHT_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_RELAY_RIGHT_TARGET_X,
    y: AGENT_GYM_RELAY_TARGET_Y,
};

pub const AGENT_GYM_KEY_GATE_START: PixelPoint = PixelPoint {
    x: AGENT_GYM_KEY_GATE_START_X,
    y: AGENT_GYM_KEY_GATE_START_Y,
};

pub const AGENT_GYM_KEY_GATE_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_KEY_GATE_TARGET_X,
    y: AGENT_GYM_KEY_GATE_TARGET_Y,
};

pub const AGENT_GYM_POWER_CHAIN_START: PixelPoint = PixelPoint {
    x: AGENT_GYM_POWER_CHAIN_START_X,
    y: AGENT_GYM_POWER_CHAIN_START_Y,
};

pub const AGENT_GYM_POWER_CHAIN_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_POWER_CHAIN_TARGET_X,
    y: AGENT_GYM_POWER_CHAIN_TARGET_Y,
};

pub const AGENT_GYM_BRANCH_SELECTOR_START: PixelPoint = PixelPoint {
    x: AGENT_GYM_BRANCH_SELECTOR_START_X,
    y: AGENT_GYM_BRANCH_SELECTOR_START_Y,
};

pub const AGENT_GYM_BRANCH_SELECTOR_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_BRANCH_SELECTOR_TARGET_X,
    y: AGENT_GYM_BRANCH_SELECTOR_TARGET_Y,
};

pub const AGENT_GYM_NESTED_START: PixelPoint = PixelPoint {
    x: AGENT_GYM_NESTED_START_X,
    y: AGENT_GYM_NESTED_START_Y,
};

pub const AGENT_GYM_NESTED_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_NESTED_TARGET_X,
    y: AGENT_GYM_NESTED_TARGET_Y,
};

pub const AGENT_GYM_BINDING_START: PixelPoint = PixelPoint {
    x: AGENT_GYM_BINDING_START_X,
    y: AGENT_GYM_BINDING_START_Y,
};

pub const AGENT_GYM_BINDING_LEFT_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_BINDING_LEFT_TARGET_X,
    y: AGENT_GYM_BINDING_TARGET_Y,
};

pub const AGENT_GYM_BINDING_RIGHT_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_BINDING_RIGHT_TARGET_X,
    y: AGENT_GYM_BINDING_TARGET_Y,
};

pub static AGENT_GYM_ORACLE: [OracleLeg; 2] = [
    OracleLeg {
        button: "RIGHT",
        frames: 60,
    },
    OracleLeg {
        button: "DOWN",
        frames: 44,
    },
];

pub static AGENT_GYM_MIRROR_ORACLE: [OracleLeg; 2] = [
    OracleLeg {
        button: "LEFT",
        frames: 60,
    },
    OracleLeg {
        button: "UP",
        frames: 44,
    },
];

pub static AGENT_GYM_WALL_ORACLE: [OracleLeg; 3] = [
    OracleLeg {
        button: "DOWN",
        frames: 40,
    },
    OracleLeg {
        button: "RIGHT",
        frames: 60,
    },
    OracleLeg {
        button: "UP",
        frames: 40,
    },
];

pub static AGENT_GYM_TEMPORAL_LEFT_ORACLE: [OracleLeg; 3] = [
    OracleLeg {
        button: "A",
        frames: 1,
    },
    OracleLeg {
        button: "WAIT",
        frames: 101,
    },
    OracleLeg {
        button: "LEFT",
        frames: 24,
    },
];

pub static AGENT_GYM_TEMPORAL_RIGHT_ORACLE: [OracleLeg; 3] = [
    OracleLeg {
        button: "A",
        frames: 1,
    },
    OracleLeg {
        button: "WAIT",
        frames: 101,
    },
    OracleLeg {
        button: "RIGHT",
        frames: 24,
    },
];

pub static AGENT_GYM_BINDING_LEFT_ORACLE: [OracleLeg; 4] = [
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 101 },
    OracleLeg { button: "LEFT", frames: 24 },
    OracleLeg { button: "A", frames: 1 },
];

pub static AGENT_GYM_BINDING_RIGHT_ORACLE: [OracleLeg; 4] = [
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 101 },
    OracleLeg { button: "RIGHT", frames: 24 },
    OracleLeg { button: "A", frames: 1 },
];

pub static AGENT_GYM_SEQUENTIAL_LEFT_ORACLE: [OracleLeg; 6] = [
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 101 },
    OracleLeg { button: "A", frames: 2 },
    OracleLeg { button: "WAIT", frames: 101 },
    OracleLeg { button: "LEFT", frames: 24 },
    OracleLeg { button: "A", frames: 1 },
];

pub static AGENT_GYM_SEQUENTIAL_RIGHT_ORACLE: [OracleLeg; 6] = [
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 101 },
    OracleLeg { button: "A", frames: 2 },
    OracleLeg { button: "WAIT", frames: 101 },
    OracleLeg { button: "RIGHT", frames: 24 },
    OracleLeg { button: "A", frames: 1 },
];

pub static AGENT_GYM_RELAY_LEFT_ORACLE: [OracleLeg; 7] = [
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "DOWN", frames: 40 },
    OracleLeg { button: "RIGHT", frames: 60 },
    OracleLeg { button: "UP", frames: 40 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "LEFT", frames: 24 },
];

pub static AGENT_GYM_RELAY_RIGHT_ORACLE: [OracleLeg; 7] = [
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "DOWN", frames: 40 },
    OracleLeg { button: "RIGHT", frames: 60 },
    OracleLeg { button: "UP", frames: 40 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "RIGHT", frames: 24 },
];

pub static AGENT_GYM_KEY_GATE_LEFT_ORACLE: [OracleLeg; 6] = [
    OracleLeg { button: "LEFT", frames: 28 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "RIGHT", frames: 28 },
    OracleLeg { button: "UP", frames: 20 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "UP", frames: 32 },
];

pub static AGENT_GYM_KEY_GATE_RIGHT_ORACLE: [OracleLeg; 6] = [
    OracleLeg { button: "RIGHT", frames: 28 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "LEFT", frames: 28 },
    OracleLeg { button: "UP", frames: 20 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "UP", frames: 32 },
];

pub static AGENT_GYM_POWER_CHAIN_LEFT_ORACLE: [OracleLeg; 10] = [
    OracleLeg { button: "LEFT", frames: 28 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "RIGHT", frames: 28 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 20 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 32 },
];

pub static AGENT_GYM_POWER_CHAIN_RIGHT_ORACLE: [OracleLeg; 10] = [
    OracleLeg { button: "RIGHT", frames: 28 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "LEFT", frames: 28 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 20 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 32 },
];

pub static AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_ORACLE: [OracleLeg; 10] = [
    OracleLeg { button: "LEFT", frames: 28 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "RIGHT", frames: 28 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 20 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 32 },
];

pub static AGENT_GYM_BRANCH_SELECTOR_SQUARE_ORACLE: [OracleLeg; 10] = [
    OracleLeg { button: "RIGHT", frames: 28 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "LEFT", frames: 28 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 20 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 32 },
];

pub static AGENT_GYM_NESTED_TRIANGLE_CIRCLE_ORACLE: [OracleLeg; 16] = [
    OracleLeg { button: "LEFT", frames: 32 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "RIGHT", frames: 32 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 16 },
    OracleLeg { button: "LEFT", frames: 32 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "RIGHT", frames: 32 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 12 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 20 },
];

pub static AGENT_GYM_NESTED_TRIANGLE_CROSS_ORACLE: [OracleLeg; 16] = [
    OracleLeg { button: "LEFT", frames: 32 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "RIGHT", frames: 32 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 16 },
    OracleLeg { button: "RIGHT", frames: 32 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "LEFT", frames: 32 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 12 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 20 },
];

pub static AGENT_GYM_NESTED_SQUARE_CIRCLE_ORACLE: [OracleLeg; 16] = [
    OracleLeg { button: "RIGHT", frames: 32 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "LEFT", frames: 32 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 16 },
    OracleLeg { button: "LEFT", frames: 32 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "RIGHT", frames: 32 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 12 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 20 },
];

pub static AGENT_GYM_NESTED_SQUARE_CROSS_ORACLE: [OracleLeg; 16] = [
    OracleLeg { button: "RIGHT", frames: 32 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "LEFT", frames: 32 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 16 },
    OracleLeg { button: "RIGHT", frames: 32 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "LEFT", frames: 32 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 12 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "A", frames: 1 },
    OracleLeg { button: "WAIT", frames: 2 },
    OracleLeg { button: "UP", frames: 20 },
];

pub const AGENT_GYM_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V1_ID,
    id: AGENT_GYM_ID,
    title: "Move the Block to the X",
    rom_sha256: AGENT_GYM_ROM_SHA256,
    source_sha256: AGENT_GYM_SOURCE_SHA256,
    start: AGENT_GYM_START,
    target: AGENT_GYM_TARGET,
    initial_distance: AGENT_GYM_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_DPAD_BUTTONS,
    prompt: "Benchmark task: move the solid square block onto the visible X target using the D-pad.",
    oracle: &AGENT_GYM_ORACLE,
};

pub const AGENT_GYM_MIRROR_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V1_ID,
    id: AGENT_GYM_MIRROR_ID,
    title: "Mirror Dash",
    rom_sha256: AGENT_GYM_MIRROR_ROM_SHA256,
    source_sha256: AGENT_GYM_MIRROR_SOURCE_SHA256,
    start: AGENT_GYM_MIRROR_START,
    target: AGENT_GYM_MIRROR_TARGET,
    initial_distance: AGENT_GYM_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_DPAD_BUTTONS,
    prompt: "Benchmark task: move the solid square block onto the visible X target using the D-pad.",
    oracle: &AGENT_GYM_MIRROR_ORACLE,
};

pub const AGENT_GYM_WALL_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V2_ID,
    id: AGENT_GYM_WALL_ID,
    title: "Wall Detour",
    rom_sha256: AGENT_GYM_WALL_ROM_SHA256,
    source_sha256: AGENT_GYM_WALL_SOURCE_SHA256,
    start: AGENT_GYM_WALL_START,
    target: AGENT_GYM_WALL_TARGET,
    initial_distance: AGENT_GYM_WALL_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_DPAD_BUTTONS,
    prompt: "Benchmark task: move the solid square block onto the visible X target using the D-pad. Navigate around visible obstacles.",
    oracle: &AGENT_GYM_WALL_ORACLE,
};

pub const AGENT_GYM_TEMPORAL_LEFT_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V3_ID,
    id: AGENT_GYM_TEMPORAL_LEFT_ID,
    title: "Temporal Cue: Left",
    rom_sha256: AGENT_GYM_TEMPORAL_LEFT_ROM_SHA256,
    source_sha256: AGENT_GYM_TEMPORAL_LEFT_SOURCE_SHA256,
    start: AGENT_GYM_TEMPORAL_START,
    target: AGENT_GYM_TEMPORAL_LEFT_TARGET,
    initial_distance: AGENT_GYM_TEMPORAL_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_TEMPORAL_BUTTONS,
    prompt: "Benchmark task: memorize the visible arrow cue, press A to dismiss it, wait for the choice chamber to unlock, then move the solid block through the side indicated by the earlier cue. The choice screen intentionally does not repeat the cue.",
    oracle: &AGENT_GYM_TEMPORAL_LEFT_ORACLE,
};

pub const AGENT_GYM_TEMPORAL_RIGHT_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V3_ID,
    id: AGENT_GYM_TEMPORAL_RIGHT_ID,
    title: "Temporal Cue: Right",
    rom_sha256: AGENT_GYM_TEMPORAL_RIGHT_ROM_SHA256,
    source_sha256: AGENT_GYM_TEMPORAL_RIGHT_SOURCE_SHA256,
    start: AGENT_GYM_TEMPORAL_START,
    target: AGENT_GYM_TEMPORAL_RIGHT_TARGET,
    initial_distance: AGENT_GYM_TEMPORAL_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_TEMPORAL_BUTTONS,
    prompt: "Benchmark task: memorize the visible arrow cue, press A to dismiss it, wait for the choice chamber to unlock, then move the solid block through the side indicated by the earlier cue. The choice screen intentionally does not repeat the cue.",
    oracle: &AGENT_GYM_TEMPORAL_RIGHT_ORACLE,
};

pub const AGENT_GYM_RELAY_LEFT_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V4_ID,
    id: AGENT_GYM_RELAY_LEFT_ID,
    title: "Relay Rooms: Left",
    rom_sha256: AGENT_GYM_RELAY_LEFT_ROM_SHA256,
    source_sha256: AGENT_GYM_RELAY_LEFT_SOURCE_SHA256,
    start: AGENT_GYM_RELAY_START,
    target: AGENT_GYM_RELAY_LEFT_TARGET,
    initial_distance: AGENT_GYM_RELAY_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_RELAY_BUTTONS,
    prompt: "Benchmark task: remember the briefing arrow, press A to leave the briefing room, navigate the visible corridor around the wall to its exit, then in the terminal room move the solid block to the side indicated by the earlier briefing. The corridor and terminal do not repeat the briefing cue.",
    oracle: &AGENT_GYM_RELAY_LEFT_ORACLE,
};

pub const AGENT_GYM_RELAY_RIGHT_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V4_ID,
    id: AGENT_GYM_RELAY_RIGHT_ID,
    title: "Relay Rooms: Right",
    rom_sha256: AGENT_GYM_RELAY_RIGHT_ROM_SHA256,
    source_sha256: AGENT_GYM_RELAY_RIGHT_SOURCE_SHA256,
    start: AGENT_GYM_RELAY_START,
    target: AGENT_GYM_RELAY_RIGHT_TARGET,
    initial_distance: AGENT_GYM_RELAY_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_RELAY_BUTTONS,
    prompt: "Benchmark task: remember the briefing arrow, press A to leave the briefing room, navigate the visible corridor around the wall to its exit, then in the terminal room move the solid block to the side indicated by the earlier briefing. The corridor and terminal do not repeat the briefing cue.",
    oracle: &AGENT_GYM_RELAY_RIGHT_ORACLE,
};

pub const AGENT_GYM_KEY_GATE_LEFT_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V5_ID,
    id: AGENT_GYM_KEY_GATE_LEFT_ID,
    title: "Key Gate: Left",
    rom_sha256: AGENT_GYM_KEY_GATE_LEFT_ROM_SHA256,
    source_sha256: AGENT_GYM_KEY_GATE_LEFT_SOURCE_SHA256,
    start: AGENT_GYM_KEY_GATE_START,
    target: AGENT_GYM_KEY_GATE_TARGET,
    initial_distance: AGENT_GYM_KEY_GATE_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_KEY_GATE_BUTTONS,
    prompt: "Benchmark task: find the visible key and press A while standing on it to acquire it, return to the locked central gate, press A at the gate to unlock it, then move the solid block onto the visible X target. The key may be on either side; use current pixels rather than assuming a side.",
    oracle: &AGENT_GYM_KEY_GATE_LEFT_ORACLE,
};

pub const AGENT_GYM_KEY_GATE_RIGHT_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V5_ID,
    id: AGENT_GYM_KEY_GATE_RIGHT_ID,
    title: "Key Gate: Right",
    rom_sha256: AGENT_GYM_KEY_GATE_RIGHT_ROM_SHA256,
    source_sha256: AGENT_GYM_KEY_GATE_RIGHT_SOURCE_SHA256,
    start: AGENT_GYM_KEY_GATE_START,
    target: AGENT_GYM_KEY_GATE_TARGET,
    initial_distance: AGENT_GYM_KEY_GATE_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_KEY_GATE_BUTTONS,
    prompt: "Benchmark task: find the visible key and press A while standing on it to acquire it, return to the locked central gate, press A at the gate to unlock it, then move the solid block onto the visible X target. The key may be on either side; use current pixels rather than assuming a side.",
    oracle: &AGENT_GYM_KEY_GATE_RIGHT_ORACLE,
};

pub const AGENT_GYM_POWER_CHAIN_LEFT_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V6_ID,
    id: AGENT_GYM_POWER_CHAIN_LEFT_ID,
    title: "Power Chain: Left",
    rom_sha256: AGENT_GYM_POWER_CHAIN_LEFT_ROM_SHA256,
    source_sha256: AGENT_GYM_POWER_CHAIN_LEFT_SOURCE_SHA256,
    start: AGENT_GYM_POWER_CHAIN_START,
    target: AGENT_GYM_POWER_CHAIN_TARGET,
    initial_distance: AGENT_GYM_POWER_CHAIN_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_POWER_CHAIN_BUTTONS,
    prompt: "Benchmark task: find the visible fuse and press A while standing on it, return to the central generator and press A to install the fuse and power the system, then move to the locked central gate, press A to open the powered gate, and reach the visible X target. The fuse may be on either side; follow the visible state changes in order.",
    oracle: &AGENT_GYM_POWER_CHAIN_LEFT_ORACLE,
};

pub const AGENT_GYM_POWER_CHAIN_RIGHT_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V6_ID,
    id: AGENT_GYM_POWER_CHAIN_RIGHT_ID,
    title: "Power Chain: Right",
    rom_sha256: AGENT_GYM_POWER_CHAIN_RIGHT_ROM_SHA256,
    source_sha256: AGENT_GYM_POWER_CHAIN_RIGHT_SOURCE_SHA256,
    start: AGENT_GYM_POWER_CHAIN_START,
    target: AGENT_GYM_POWER_CHAIN_TARGET,
    initial_distance: AGENT_GYM_POWER_CHAIN_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_POWER_CHAIN_BUTTONS,
    prompt: "Benchmark task: find the visible fuse and press A while standing on it, return to the central generator and press A to install the fuse and power the system, then move to the locked central gate, press A to open the powered gate, and reach the visible X target. The fuse may be on either side; follow the visible state changes in order.",
    oracle: &AGENT_GYM_POWER_CHAIN_RIGHT_ORACLE,
};

pub const AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V7_ID,
    id: AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_ID,
    title: "Branch Selector: Triangle",
    rom_sha256: AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_ROM_SHA256,
    source_sha256: AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_SOURCE_SHA256,
    start: AGENT_GYM_BRANCH_SELECTOR_START,
    target: AGENT_GYM_BRANCH_SELECTOR_TARGET,
    initial_distance: AGENT_GYM_BRANCH_SELECTOR_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_BRANCH_SELECTOR_BUTTONS,
    prompt: "Benchmark task: both a triangle module on the left and a square module on the right are visible. Match the central selector symbol to the same-shaped module and press A on that module. Choosing the wrong module enters an irreversible fail state. Return to the center generator, press A to install the accepted module and power the system, then move to the gate, press A to open it, and reach the visible X target.",
    oracle: &AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_ORACLE,
};

pub const AGENT_GYM_BRANCH_SELECTOR_SQUARE_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V7_ID,
    id: AGENT_GYM_BRANCH_SELECTOR_SQUARE_ID,
    title: "Branch Selector: Square",
    rom_sha256: AGENT_GYM_BRANCH_SELECTOR_SQUARE_ROM_SHA256,
    source_sha256: AGENT_GYM_BRANCH_SELECTOR_SQUARE_SOURCE_SHA256,
    start: AGENT_GYM_BRANCH_SELECTOR_START,
    target: AGENT_GYM_BRANCH_SELECTOR_TARGET,
    initial_distance: AGENT_GYM_BRANCH_SELECTOR_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_BRANCH_SELECTOR_BUTTONS,
    prompt: "Benchmark task: both a triangle module on the left and a square module on the right are visible. Match the central selector symbol to the same-shaped module and press A on that module. Choosing the wrong module enters an irreversible fail state. Return to the center generator, press A to install the accepted module and power the system, then move to the gate, press A to open it, and reach the visible X target.",
    oracle: &AGENT_GYM_BRANCH_SELECTOR_SQUARE_ORACLE,
};

pub const AGENT_GYM_NESTED_TRIANGLE_CIRCLE_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V8_ID,
    id: AGENT_GYM_NESTED_TRIANGLE_CIRCLE_ID,
    title: "Nested Branch: Triangle → Circle",
    rom_sha256: AGENT_GYM_NESTED_TRIANGLE_CIRCLE_ROM_SHA256,
    source_sha256: AGENT_GYM_NESTED_TRIANGLE_CIRCLE_SOURCE_SHA256,
    start: AGENT_GYM_NESTED_START,
    target: AGENT_GYM_NESTED_TARGET,
    initial_distance: AGENT_GYM_NESTED_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_NESTED_BUTTONS,
    prompt: "Benchmark task: solve two selector stages in order. Stage 1 always shows a triangle family on the left and a square family on the right; match the visible Stage 1 selector and press A on that family. Only after a correct Stage 1 commitment will Stage 2 appear, showing a circle submodule on the left and a cross submodule on the right. Match the newly revealed Stage 2 selector and press A on that submodule. A wrong commitment at either stage is irreversible. After both correct choices, return to the shared center generator, press A to power it, move to the gate, press A to open it, and reach the visible X target.",
    oracle: &AGENT_GYM_NESTED_TRIANGLE_CIRCLE_ORACLE,
};

pub const AGENT_GYM_NESTED_TRIANGLE_CROSS_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V8_ID,
    id: AGENT_GYM_NESTED_TRIANGLE_CROSS_ID,
    title: "Nested Branch: Triangle → Cross",
    rom_sha256: AGENT_GYM_NESTED_TRIANGLE_CROSS_ROM_SHA256,
    source_sha256: AGENT_GYM_NESTED_TRIANGLE_CROSS_SOURCE_SHA256,
    start: AGENT_GYM_NESTED_START,
    target: AGENT_GYM_NESTED_TARGET,
    initial_distance: AGENT_GYM_NESTED_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_NESTED_BUTTONS,
    prompt: "Benchmark task: solve two selector stages in order. Stage 1 always shows a triangle family on the left and a square family on the right; match the visible Stage 1 selector and press A on that family. Only after a correct Stage 1 commitment will Stage 2 appear, showing a circle submodule on the left and a cross submodule on the right. Match the newly revealed Stage 2 selector and press A on that submodule. A wrong commitment at either stage is irreversible. After both correct choices, return to the shared center generator, press A to power it, move to the gate, press A to open it, and reach the visible X target.",
    oracle: &AGENT_GYM_NESTED_TRIANGLE_CROSS_ORACLE,
};

pub const AGENT_GYM_NESTED_SQUARE_CIRCLE_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V8_ID,
    id: AGENT_GYM_NESTED_SQUARE_CIRCLE_ID,
    title: "Nested Branch: Square → Circle",
    rom_sha256: AGENT_GYM_NESTED_SQUARE_CIRCLE_ROM_SHA256,
    source_sha256: AGENT_GYM_NESTED_SQUARE_CIRCLE_SOURCE_SHA256,
    start: AGENT_GYM_NESTED_START,
    target: AGENT_GYM_NESTED_TARGET,
    initial_distance: AGENT_GYM_NESTED_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_NESTED_BUTTONS,
    prompt: "Benchmark task: solve two selector stages in order. Stage 1 always shows a triangle family on the left and a square family on the right; match the visible Stage 1 selector and press A on that family. Only after a correct Stage 1 commitment will Stage 2 appear, showing a circle submodule on the left and a cross submodule on the right. Match the newly revealed Stage 2 selector and press A on that submodule. A wrong commitment at either stage is irreversible. After both correct choices, return to the shared center generator, press A to power it, move to the gate, press A to open it, and reach the visible X target.",
    oracle: &AGENT_GYM_NESTED_SQUARE_CIRCLE_ORACLE,
};

pub const AGENT_GYM_NESTED_SQUARE_CROSS_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V8_ID,
    id: AGENT_GYM_NESTED_SQUARE_CROSS_ID,
    title: "Nested Branch: Square → Cross",
    rom_sha256: AGENT_GYM_NESTED_SQUARE_CROSS_ROM_SHA256,
    source_sha256: AGENT_GYM_NESTED_SQUARE_CROSS_SOURCE_SHA256,
    start: AGENT_GYM_NESTED_START,
    target: AGENT_GYM_NESTED_TARGET,
    initial_distance: AGENT_GYM_NESTED_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_NESTED_BUTTONS,
    prompt: "Benchmark task: solve two selector stages in order. Stage 1 always shows a triangle family on the left and a square family on the right; match the visible Stage 1 selector and press A on that family. Only after a correct Stage 1 commitment will Stage 2 appear, showing a circle submodule on the left and a cross submodule on the right. Match the newly revealed Stage 2 selector and press A on that submodule. A wrong commitment at either stage is irreversible. After both correct choices, return to the shared center generator, press A to power it, move to the gate, press A to open it, and reach the visible X target.",
    oracle: &AGENT_GYM_NESTED_SQUARE_CROSS_ORACLE,
};

pub const AGENT_GYM_BINDING_NORMAL_TRIANGLE_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V9_ID,
    id: AGENT_GYM_BINDING_NORMAL_TRIANGLE_ID,
    title: "Binding Memory: Normal / Triangle",
    rom_sha256: AGENT_GYM_BINDING_NORMAL_TRIANGLE_ROM_SHA256,
    source_sha256: AGENT_GYM_BINDING_NORMAL_TRIANGLE_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_LEFT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_BINDING_BUTTONS,
    prompt: "Benchmark task: memorize where TRIANGLE and SQUARE appear in the briefing, press A to erase the briefing, wait for the choice chamber to unlock, then use the newly shown query symbol to choose the door where that symbol appeared earlier and press A to commit. The two doors are identical and a wrong commitment is irreversible.",
    oracle: &AGENT_GYM_BINDING_LEFT_ORACLE,
};

pub const AGENT_GYM_BINDING_NORMAL_SQUARE_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V9_ID,
    id: AGENT_GYM_BINDING_NORMAL_SQUARE_ID,
    title: "Binding Memory: Normal / Square",
    rom_sha256: AGENT_GYM_BINDING_NORMAL_SQUARE_ROM_SHA256,
    source_sha256: AGENT_GYM_BINDING_NORMAL_SQUARE_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_RIGHT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_BINDING_BUTTONS,
    prompt: AGENT_GYM_BINDING_NORMAL_TRIANGLE_TASK.prompt,
    oracle: &AGENT_GYM_BINDING_RIGHT_ORACLE,
};

pub const AGENT_GYM_BINDING_SWAPPED_TRIANGLE_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V9_ID,
    id: AGENT_GYM_BINDING_SWAPPED_TRIANGLE_ID,
    title: "Binding Memory: Swapped / Triangle",
    rom_sha256: AGENT_GYM_BINDING_SWAPPED_TRIANGLE_ROM_SHA256,
    source_sha256: AGENT_GYM_BINDING_SWAPPED_TRIANGLE_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_RIGHT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_BINDING_BUTTONS,
    prompt: AGENT_GYM_BINDING_NORMAL_TRIANGLE_TASK.prompt,
    oracle: &AGENT_GYM_BINDING_RIGHT_ORACLE,
};

pub const AGENT_GYM_BINDING_SWAPPED_SQUARE_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V9_ID,
    id: AGENT_GYM_BINDING_SWAPPED_SQUARE_ID,
    title: "Binding Memory: Swapped / Square",
    rom_sha256: AGENT_GYM_BINDING_SWAPPED_SQUARE_ROM_SHA256,
    source_sha256: AGENT_GYM_BINDING_SWAPPED_SQUARE_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_LEFT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_BINDING_BUTTONS,
    prompt: AGENT_GYM_BINDING_NORMAL_TRIANGLE_TASK.prompt,
    oracle: &AGENT_GYM_BINDING_LEFT_ORACLE,
};

pub const AGENT_GYM_COMP_NTM_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V10_ID,
    id: AGENT_GYM_COMP_NTM_ID,
    title: "Compositional Recall: Normal / Triangle / MATCH",
    rom_sha256: AGENT_GYM_COMP_NTM_ROM_SHA256,
    source_sha256: AGENT_GYM_COMP_NTM_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_LEFT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_COMPOSITIONAL_BUTTONS,
    prompt: "Benchmark task: memorize where TRIANGLE and SQUARE appear in the briefing, press A to erase the briefing, and wait for the choice chamber. The later scene shows a query symbol plus an operator. MATCH (=) means choose the door where that queried symbol appeared earlier; FLIP (X) means choose the opposite door. The doors are identical and a wrong commitment is irreversible.",
    oracle: &AGENT_GYM_BINDING_LEFT_ORACLE,
};

pub const AGENT_GYM_COMP_NTF_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V10_ID,
    id: AGENT_GYM_COMP_NTF_ID,
    title: "Compositional Recall: Normal / Triangle / FLIP",
    rom_sha256: AGENT_GYM_COMP_NTF_ROM_SHA256,
    source_sha256: AGENT_GYM_COMP_NTF_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_RIGHT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_COMPOSITIONAL_BUTTONS,
    prompt: AGENT_GYM_COMP_NTM_TASK.prompt,
    oracle: &AGENT_GYM_BINDING_RIGHT_ORACLE,
};

pub const AGENT_GYM_COMP_NSM_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V10_ID,
    id: AGENT_GYM_COMP_NSM_ID,
    title: "Compositional Recall: Normal / Square / MATCH",
    rom_sha256: AGENT_GYM_COMP_NSM_ROM_SHA256,
    source_sha256: AGENT_GYM_COMP_NSM_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_RIGHT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_COMPOSITIONAL_BUTTONS,
    prompt: AGENT_GYM_COMP_NTM_TASK.prompt,
    oracle: &AGENT_GYM_BINDING_RIGHT_ORACLE,
};

pub const AGENT_GYM_COMP_NSF_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V10_ID,
    id: AGENT_GYM_COMP_NSF_ID,
    title: "Compositional Recall: Normal / Square / FLIP",
    rom_sha256: AGENT_GYM_COMP_NSF_ROM_SHA256,
    source_sha256: AGENT_GYM_COMP_NSF_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_LEFT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_COMPOSITIONAL_BUTTONS,
    prompt: AGENT_GYM_COMP_NTM_TASK.prompt,
    oracle: &AGENT_GYM_BINDING_LEFT_ORACLE,
};

pub const AGENT_GYM_COMP_STM_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V10_ID,
    id: AGENT_GYM_COMP_STM_ID,
    title: "Compositional Recall: Swapped / Triangle / MATCH",
    rom_sha256: AGENT_GYM_COMP_STM_ROM_SHA256,
    source_sha256: AGENT_GYM_COMP_STM_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_RIGHT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_COMPOSITIONAL_BUTTONS,
    prompt: AGENT_GYM_COMP_NTM_TASK.prompt,
    oracle: &AGENT_GYM_BINDING_RIGHT_ORACLE,
};

pub const AGENT_GYM_COMP_STF_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V10_ID,
    id: AGENT_GYM_COMP_STF_ID,
    title: "Compositional Recall: Swapped / Triangle / FLIP",
    rom_sha256: AGENT_GYM_COMP_STF_ROM_SHA256,
    source_sha256: AGENT_GYM_COMP_STF_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_LEFT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_COMPOSITIONAL_BUTTONS,
    prompt: AGENT_GYM_COMP_NTM_TASK.prompt,
    oracle: &AGENT_GYM_BINDING_LEFT_ORACLE,
};

pub const AGENT_GYM_COMP_SSM_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V10_ID,
    id: AGENT_GYM_COMP_SSM_ID,
    title: "Compositional Recall: Swapped / Square / MATCH",
    rom_sha256: AGENT_GYM_COMP_SSM_ROM_SHA256,
    source_sha256: AGENT_GYM_COMP_SSM_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_LEFT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_COMPOSITIONAL_BUTTONS,
    prompt: AGENT_GYM_COMP_NTM_TASK.prompt,
    oracle: &AGENT_GYM_BINDING_LEFT_ORACLE,
};

pub const AGENT_GYM_COMP_SSF_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V10_ID,
    id: AGENT_GYM_COMP_SSF_ID,
    title: "Compositional Recall: Swapped / Square / FLIP",
    rom_sha256: AGENT_GYM_COMP_SSF_ROM_SHA256,
    source_sha256: AGENT_GYM_COMP_SSF_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_RIGHT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_COMPOSITIONAL_BUTTONS,
    prompt: AGENT_GYM_COMP_NTM_TASK.prompt,
    oracle: &AGENT_GYM_BINDING_RIGHT_ORACLE,
};

pub const AGENT_GYM_SEQ_NTMM_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V11_ID,
    id: AGENT_GYM_SEQ_NTMM_ID,
    title: "Sequential Rule: Normal / Triangle / MATCH / MATCH",
    rom_sha256: AGENT_GYM_SEQ_NTMM_ROM_SHA256,
    source_sha256: AGENT_GYM_SEQ_NTMM_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_LEFT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_SEQUENTIAL_BUTTONS,
    prompt: "Benchmark task: memorize where TRIANGLE and SQUARE appear, press A to erase the briefing, and wait. Stage 1 shows a query symbol plus operator 1. Apply MATCH (=) by preserving the remembered side or FLIP (X) by inverting it, retain that intermediate side, then press A to erase Stage 1. After the next delay, Stage 2 shows operator 2 only. Apply it to the hidden intermediate side, choose the final identical door, and press A. A wrong commitment is irreversible.",
    oracle: &AGENT_GYM_SEQUENTIAL_LEFT_ORACLE,
};

pub const AGENT_GYM_SEQ_NTMF_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V11_ID,
    id: AGENT_GYM_SEQ_NTMF_ID,
    title: "Sequential Rule: Normal / Triangle / MATCH / FLIP",
    rom_sha256: AGENT_GYM_SEQ_NTMF_ROM_SHA256,
    source_sha256: AGENT_GYM_SEQ_NTMF_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_RIGHT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_SEQUENTIAL_BUTTONS,
    prompt: AGENT_GYM_SEQ_NTMM_TASK.prompt,
    oracle: &AGENT_GYM_SEQUENTIAL_RIGHT_ORACLE,
};

pub const AGENT_GYM_SEQ_NTFM_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V11_ID,
    id: AGENT_GYM_SEQ_NTFM_ID,
    title: "Sequential Rule: Normal / Triangle / FLIP / MATCH",
    rom_sha256: AGENT_GYM_SEQ_NTFM_ROM_SHA256,
    source_sha256: AGENT_GYM_SEQ_NTFM_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_RIGHT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_SEQUENTIAL_BUTTONS,
    prompt: AGENT_GYM_SEQ_NTMM_TASK.prompt,
    oracle: &AGENT_GYM_SEQUENTIAL_RIGHT_ORACLE,
};

pub const AGENT_GYM_SEQ_NTFF_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V11_ID,
    id: AGENT_GYM_SEQ_NTFF_ID,
    title: "Sequential Rule: Normal / Triangle / FLIP / FLIP",
    rom_sha256: AGENT_GYM_SEQ_NTFF_ROM_SHA256,
    source_sha256: AGENT_GYM_SEQ_NTFF_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_LEFT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_SEQUENTIAL_BUTTONS,
    prompt: AGENT_GYM_SEQ_NTMM_TASK.prompt,
    oracle: &AGENT_GYM_SEQUENTIAL_LEFT_ORACLE,
};

pub const AGENT_GYM_SEQ_NSMM_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V11_ID,
    id: AGENT_GYM_SEQ_NSMM_ID,
    title: "Sequential Rule: Normal / Square / MATCH / MATCH",
    rom_sha256: AGENT_GYM_SEQ_NSMM_ROM_SHA256,
    source_sha256: AGENT_GYM_SEQ_NSMM_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_RIGHT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_SEQUENTIAL_BUTTONS,
    prompt: AGENT_GYM_SEQ_NTMM_TASK.prompt,
    oracle: &AGENT_GYM_SEQUENTIAL_RIGHT_ORACLE,
};

pub const AGENT_GYM_SEQ_NSMF_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V11_ID,
    id: AGENT_GYM_SEQ_NSMF_ID,
    title: "Sequential Rule: Normal / Square / MATCH / FLIP",
    rom_sha256: AGENT_GYM_SEQ_NSMF_ROM_SHA256,
    source_sha256: AGENT_GYM_SEQ_NSMF_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_LEFT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_SEQUENTIAL_BUTTONS,
    prompt: AGENT_GYM_SEQ_NTMM_TASK.prompt,
    oracle: &AGENT_GYM_SEQUENTIAL_LEFT_ORACLE,
};

pub const AGENT_GYM_SEQ_NSFM_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V11_ID,
    id: AGENT_GYM_SEQ_NSFM_ID,
    title: "Sequential Rule: Normal / Square / FLIP / MATCH",
    rom_sha256: AGENT_GYM_SEQ_NSFM_ROM_SHA256,
    source_sha256: AGENT_GYM_SEQ_NSFM_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_LEFT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_SEQUENTIAL_BUTTONS,
    prompt: AGENT_GYM_SEQ_NTMM_TASK.prompt,
    oracle: &AGENT_GYM_SEQUENTIAL_LEFT_ORACLE,
};

pub const AGENT_GYM_SEQ_NSFF_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V11_ID,
    id: AGENT_GYM_SEQ_NSFF_ID,
    title: "Sequential Rule: Normal / Square / FLIP / FLIP",
    rom_sha256: AGENT_GYM_SEQ_NSFF_ROM_SHA256,
    source_sha256: AGENT_GYM_SEQ_NSFF_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_RIGHT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_SEQUENTIAL_BUTTONS,
    prompt: AGENT_GYM_SEQ_NTMM_TASK.prompt,
    oracle: &AGENT_GYM_SEQUENTIAL_RIGHT_ORACLE,
};

pub const AGENT_GYM_SEQ_STMM_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V11_ID,
    id: AGENT_GYM_SEQ_STMM_ID,
    title: "Sequential Rule: Swapped / Triangle / MATCH / MATCH",
    rom_sha256: AGENT_GYM_SEQ_STMM_ROM_SHA256,
    source_sha256: AGENT_GYM_SEQ_STMM_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_RIGHT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_SEQUENTIAL_BUTTONS,
    prompt: AGENT_GYM_SEQ_NTMM_TASK.prompt,
    oracle: &AGENT_GYM_SEQUENTIAL_RIGHT_ORACLE,
};

pub const AGENT_GYM_SEQ_STMF_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V11_ID,
    id: AGENT_GYM_SEQ_STMF_ID,
    title: "Sequential Rule: Swapped / Triangle / MATCH / FLIP",
    rom_sha256: AGENT_GYM_SEQ_STMF_ROM_SHA256,
    source_sha256: AGENT_GYM_SEQ_STMF_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_LEFT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_SEQUENTIAL_BUTTONS,
    prompt: AGENT_GYM_SEQ_NTMM_TASK.prompt,
    oracle: &AGENT_GYM_SEQUENTIAL_LEFT_ORACLE,
};

pub const AGENT_GYM_SEQ_STFM_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V11_ID,
    id: AGENT_GYM_SEQ_STFM_ID,
    title: "Sequential Rule: Swapped / Triangle / FLIP / MATCH",
    rom_sha256: AGENT_GYM_SEQ_STFM_ROM_SHA256,
    source_sha256: AGENT_GYM_SEQ_STFM_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_LEFT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_SEQUENTIAL_BUTTONS,
    prompt: AGENT_GYM_SEQ_NTMM_TASK.prompt,
    oracle: &AGENT_GYM_SEQUENTIAL_LEFT_ORACLE,
};

pub const AGENT_GYM_SEQ_STFF_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V11_ID,
    id: AGENT_GYM_SEQ_STFF_ID,
    title: "Sequential Rule: Swapped / Triangle / FLIP / FLIP",
    rom_sha256: AGENT_GYM_SEQ_STFF_ROM_SHA256,
    source_sha256: AGENT_GYM_SEQ_STFF_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_RIGHT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_SEQUENTIAL_BUTTONS,
    prompt: AGENT_GYM_SEQ_NTMM_TASK.prompt,
    oracle: &AGENT_GYM_SEQUENTIAL_RIGHT_ORACLE,
};

pub const AGENT_GYM_SEQ_SSMM_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V11_ID,
    id: AGENT_GYM_SEQ_SSMM_ID,
    title: "Sequential Rule: Swapped / Square / MATCH / MATCH",
    rom_sha256: AGENT_GYM_SEQ_SSMM_ROM_SHA256,
    source_sha256: AGENT_GYM_SEQ_SSMM_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_LEFT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_SEQUENTIAL_BUTTONS,
    prompt: AGENT_GYM_SEQ_NTMM_TASK.prompt,
    oracle: &AGENT_GYM_SEQUENTIAL_LEFT_ORACLE,
};

pub const AGENT_GYM_SEQ_SSMF_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V11_ID,
    id: AGENT_GYM_SEQ_SSMF_ID,
    title: "Sequential Rule: Swapped / Square / MATCH / FLIP",
    rom_sha256: AGENT_GYM_SEQ_SSMF_ROM_SHA256,
    source_sha256: AGENT_GYM_SEQ_SSMF_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_RIGHT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_SEQUENTIAL_BUTTONS,
    prompt: AGENT_GYM_SEQ_NTMM_TASK.prompt,
    oracle: &AGENT_GYM_SEQUENTIAL_RIGHT_ORACLE,
};

pub const AGENT_GYM_SEQ_SSFM_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V11_ID,
    id: AGENT_GYM_SEQ_SSFM_ID,
    title: "Sequential Rule: Swapped / Square / FLIP / MATCH",
    rom_sha256: AGENT_GYM_SEQ_SSFM_ROM_SHA256,
    source_sha256: AGENT_GYM_SEQ_SSFM_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_RIGHT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_SEQUENTIAL_BUTTONS,
    prompt: AGENT_GYM_SEQ_NTMM_TASK.prompt,
    oracle: &AGENT_GYM_SEQUENTIAL_RIGHT_ORACLE,
};

pub const AGENT_GYM_SEQ_SSFF_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V11_ID,
    id: AGENT_GYM_SEQ_SSFF_ID,
    title: "Sequential Rule: Swapped / Square / FLIP / FLIP",
    rom_sha256: AGENT_GYM_SEQ_SSFF_ROM_SHA256,
    source_sha256: AGENT_GYM_SEQ_SSFF_SOURCE_SHA256,
    start: AGENT_GYM_BINDING_START,
    target: AGENT_GYM_BINDING_LEFT_TARGET,
    initial_distance: AGENT_GYM_BINDING_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_SEQUENTIAL_BUTTONS,
    prompt: AGENT_GYM_SEQ_NTMM_TASK.prompt,
    oracle: &AGENT_GYM_SEQUENTIAL_LEFT_ORACLE,
};

pub static BENCHMARK_TASKS: [BenchmarkTaskSpec; 45] = [
    AGENT_GYM_TASK,
    AGENT_GYM_MIRROR_TASK,
    AGENT_GYM_WALL_TASK,
    AGENT_GYM_TEMPORAL_LEFT_TASK,
    AGENT_GYM_TEMPORAL_RIGHT_TASK,
    AGENT_GYM_RELAY_LEFT_TASK,
    AGENT_GYM_RELAY_RIGHT_TASK,
    AGENT_GYM_KEY_GATE_LEFT_TASK,
    AGENT_GYM_KEY_GATE_RIGHT_TASK,
    AGENT_GYM_POWER_CHAIN_LEFT_TASK,
    AGENT_GYM_POWER_CHAIN_RIGHT_TASK,
    AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_TASK,
    AGENT_GYM_BRANCH_SELECTOR_SQUARE_TASK,
    AGENT_GYM_NESTED_TRIANGLE_CIRCLE_TASK,
    AGENT_GYM_NESTED_TRIANGLE_CROSS_TASK,
    AGENT_GYM_NESTED_SQUARE_CIRCLE_TASK,
    AGENT_GYM_NESTED_SQUARE_CROSS_TASK,
    AGENT_GYM_BINDING_NORMAL_TRIANGLE_TASK,
    AGENT_GYM_BINDING_NORMAL_SQUARE_TASK,
    AGENT_GYM_BINDING_SWAPPED_TRIANGLE_TASK,
    AGENT_GYM_BINDING_SWAPPED_SQUARE_TASK,
    AGENT_GYM_COMP_NTM_TASK,
    AGENT_GYM_COMP_NTF_TASK,
    AGENT_GYM_COMP_NSM_TASK,
    AGENT_GYM_COMP_NSF_TASK,
    AGENT_GYM_COMP_STM_TASK,
    AGENT_GYM_COMP_STF_TASK,
    AGENT_GYM_COMP_SSM_TASK,
    AGENT_GYM_COMP_SSF_TASK,
    AGENT_GYM_SEQ_NTMM_TASK,
    AGENT_GYM_SEQ_NTMF_TASK,
    AGENT_GYM_SEQ_NTFM_TASK,
    AGENT_GYM_SEQ_NTFF_TASK,
    AGENT_GYM_SEQ_NSMM_TASK,
    AGENT_GYM_SEQ_NSMF_TASK,
    AGENT_GYM_SEQ_NSFM_TASK,
    AGENT_GYM_SEQ_NSFF_TASK,
    AGENT_GYM_SEQ_STMM_TASK,
    AGENT_GYM_SEQ_STMF_TASK,
    AGENT_GYM_SEQ_STFM_TASK,
    AGENT_GYM_SEQ_STFF_TASK,
    AGENT_GYM_SEQ_SSMM_TASK,
    AGENT_GYM_SEQ_SSMF_TASK,
    AGENT_GYM_SEQ_SSFM_TASK,
    AGENT_GYM_SEQ_SSFF_TASK,
];

pub static BENCHMARK_SUITE_V1_TASKS: [BenchmarkTaskSpec; 2] =
    [AGENT_GYM_TASK, AGENT_GYM_MIRROR_TASK];

pub static BENCHMARK_SUITE_V2_TASKS: [BenchmarkTaskSpec; 3] =
    [AGENT_GYM_TASK, AGENT_GYM_MIRROR_TASK, AGENT_GYM_WALL_TASK];

pub static BENCHMARK_SUITE_V3_TASKS: [BenchmarkTaskSpec; 5] = [
    AGENT_GYM_TASK,
    AGENT_GYM_MIRROR_TASK,
    AGENT_GYM_WALL_TASK,
    AGENT_GYM_TEMPORAL_LEFT_TASK,
    AGENT_GYM_TEMPORAL_RIGHT_TASK,
];

pub static BENCHMARK_SUITE_V4_TASKS: [BenchmarkTaskSpec; 7] = [
    AGENT_GYM_TASK,
    AGENT_GYM_MIRROR_TASK,
    AGENT_GYM_WALL_TASK,
    AGENT_GYM_TEMPORAL_LEFT_TASK,
    AGENT_GYM_TEMPORAL_RIGHT_TASK,
    AGENT_GYM_RELAY_LEFT_TASK,
    AGENT_GYM_RELAY_RIGHT_TASK,
];

pub static BENCHMARK_SUITE_V5_TASKS: [BenchmarkTaskSpec; 9] = [
    AGENT_GYM_TASK,
    AGENT_GYM_MIRROR_TASK,
    AGENT_GYM_WALL_TASK,
    AGENT_GYM_TEMPORAL_LEFT_TASK,
    AGENT_GYM_TEMPORAL_RIGHT_TASK,
    AGENT_GYM_RELAY_LEFT_TASK,
    AGENT_GYM_RELAY_RIGHT_TASK,
    AGENT_GYM_KEY_GATE_LEFT_TASK,
    AGENT_GYM_KEY_GATE_RIGHT_TASK,
];

pub static BENCHMARK_SUITE_V6_TASKS: [BenchmarkTaskSpec; 11] = [
    AGENT_GYM_TASK,
    AGENT_GYM_MIRROR_TASK,
    AGENT_GYM_WALL_TASK,
    AGENT_GYM_TEMPORAL_LEFT_TASK,
    AGENT_GYM_TEMPORAL_RIGHT_TASK,
    AGENT_GYM_RELAY_LEFT_TASK,
    AGENT_GYM_RELAY_RIGHT_TASK,
    AGENT_GYM_KEY_GATE_LEFT_TASK,
    AGENT_GYM_KEY_GATE_RIGHT_TASK,
    AGENT_GYM_POWER_CHAIN_LEFT_TASK,
    AGENT_GYM_POWER_CHAIN_RIGHT_TASK,
];

pub static BENCHMARK_SUITE_V7_TASKS: [BenchmarkTaskSpec; 13] = [
    AGENT_GYM_TASK,
    AGENT_GYM_MIRROR_TASK,
    AGENT_GYM_WALL_TASK,
    AGENT_GYM_TEMPORAL_LEFT_TASK,
    AGENT_GYM_TEMPORAL_RIGHT_TASK,
    AGENT_GYM_RELAY_LEFT_TASK,
    AGENT_GYM_RELAY_RIGHT_TASK,
    AGENT_GYM_KEY_GATE_LEFT_TASK,
    AGENT_GYM_KEY_GATE_RIGHT_TASK,
    AGENT_GYM_POWER_CHAIN_LEFT_TASK,
    AGENT_GYM_POWER_CHAIN_RIGHT_TASK,
    AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_TASK,
    AGENT_GYM_BRANCH_SELECTOR_SQUARE_TASK,
];

pub static BENCHMARK_SUITE_V8_TASKS: [BenchmarkTaskSpec; 17] = [
    AGENT_GYM_TASK,
    AGENT_GYM_MIRROR_TASK,
    AGENT_GYM_WALL_TASK,
    AGENT_GYM_TEMPORAL_LEFT_TASK,
    AGENT_GYM_TEMPORAL_RIGHT_TASK,
    AGENT_GYM_RELAY_LEFT_TASK,
    AGENT_GYM_RELAY_RIGHT_TASK,
    AGENT_GYM_KEY_GATE_LEFT_TASK,
    AGENT_GYM_KEY_GATE_RIGHT_TASK,
    AGENT_GYM_POWER_CHAIN_LEFT_TASK,
    AGENT_GYM_POWER_CHAIN_RIGHT_TASK,
    AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_TASK,
    AGENT_GYM_BRANCH_SELECTOR_SQUARE_TASK,
    AGENT_GYM_NESTED_TRIANGLE_CIRCLE_TASK,
    AGENT_GYM_NESTED_TRIANGLE_CROSS_TASK,
    AGENT_GYM_NESTED_SQUARE_CIRCLE_TASK,
    AGENT_GYM_NESTED_SQUARE_CROSS_TASK,
];

pub static BENCHMARK_SUITE_V9_TASKS: [BenchmarkTaskSpec; 21] = [
    AGENT_GYM_TASK,
    AGENT_GYM_MIRROR_TASK,
    AGENT_GYM_WALL_TASK,
    AGENT_GYM_TEMPORAL_LEFT_TASK,
    AGENT_GYM_TEMPORAL_RIGHT_TASK,
    AGENT_GYM_RELAY_LEFT_TASK,
    AGENT_GYM_RELAY_RIGHT_TASK,
    AGENT_GYM_KEY_GATE_LEFT_TASK,
    AGENT_GYM_KEY_GATE_RIGHT_TASK,
    AGENT_GYM_POWER_CHAIN_LEFT_TASK,
    AGENT_GYM_POWER_CHAIN_RIGHT_TASK,
    AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_TASK,
    AGENT_GYM_BRANCH_SELECTOR_SQUARE_TASK,
    AGENT_GYM_NESTED_TRIANGLE_CIRCLE_TASK,
    AGENT_GYM_NESTED_TRIANGLE_CROSS_TASK,
    AGENT_GYM_NESTED_SQUARE_CIRCLE_TASK,
    AGENT_GYM_NESTED_SQUARE_CROSS_TASK,
    AGENT_GYM_BINDING_NORMAL_TRIANGLE_TASK,
    AGENT_GYM_BINDING_NORMAL_SQUARE_TASK,
    AGENT_GYM_BINDING_SWAPPED_TRIANGLE_TASK,
    AGENT_GYM_BINDING_SWAPPED_SQUARE_TASK,
];

pub static BENCHMARK_SUITE_V10_TASKS: [BenchmarkTaskSpec; 29] = [
    AGENT_GYM_TASK,
    AGENT_GYM_MIRROR_TASK,
    AGENT_GYM_WALL_TASK,
    AGENT_GYM_TEMPORAL_LEFT_TASK,
    AGENT_GYM_TEMPORAL_RIGHT_TASK,
    AGENT_GYM_RELAY_LEFT_TASK,
    AGENT_GYM_RELAY_RIGHT_TASK,
    AGENT_GYM_KEY_GATE_LEFT_TASK,
    AGENT_GYM_KEY_GATE_RIGHT_TASK,
    AGENT_GYM_POWER_CHAIN_LEFT_TASK,
    AGENT_GYM_POWER_CHAIN_RIGHT_TASK,
    AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_TASK,
    AGENT_GYM_BRANCH_SELECTOR_SQUARE_TASK,
    AGENT_GYM_NESTED_TRIANGLE_CIRCLE_TASK,
    AGENT_GYM_NESTED_TRIANGLE_CROSS_TASK,
    AGENT_GYM_NESTED_SQUARE_CIRCLE_TASK,
    AGENT_GYM_NESTED_SQUARE_CROSS_TASK,
    AGENT_GYM_BINDING_NORMAL_TRIANGLE_TASK,
    AGENT_GYM_BINDING_NORMAL_SQUARE_TASK,
    AGENT_GYM_BINDING_SWAPPED_TRIANGLE_TASK,
    AGENT_GYM_BINDING_SWAPPED_SQUARE_TASK,
    AGENT_GYM_COMP_NTM_TASK,
    AGENT_GYM_COMP_NTF_TASK,
    AGENT_GYM_COMP_NSM_TASK,
    AGENT_GYM_COMP_NSF_TASK,
    AGENT_GYM_COMP_STM_TASK,
    AGENT_GYM_COMP_STF_TASK,
    AGENT_GYM_COMP_SSM_TASK,
    AGENT_GYM_COMP_SSF_TASK,
];

pub static BENCHMARK_SUITE_V11_TASKS: [BenchmarkTaskSpec; 45] = [

    AGENT_GYM_TASK,
    AGENT_GYM_MIRROR_TASK,
    AGENT_GYM_WALL_TASK,
    AGENT_GYM_TEMPORAL_LEFT_TASK,
    AGENT_GYM_TEMPORAL_RIGHT_TASK,
    AGENT_GYM_RELAY_LEFT_TASK,
    AGENT_GYM_RELAY_RIGHT_TASK,
    AGENT_GYM_KEY_GATE_LEFT_TASK,
    AGENT_GYM_KEY_GATE_RIGHT_TASK,
    AGENT_GYM_POWER_CHAIN_LEFT_TASK,
    AGENT_GYM_POWER_CHAIN_RIGHT_TASK,
    AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_TASK,
    AGENT_GYM_BRANCH_SELECTOR_SQUARE_TASK,
    AGENT_GYM_NESTED_TRIANGLE_CIRCLE_TASK,
    AGENT_GYM_NESTED_TRIANGLE_CROSS_TASK,
    AGENT_GYM_NESTED_SQUARE_CIRCLE_TASK,
    AGENT_GYM_NESTED_SQUARE_CROSS_TASK,
    AGENT_GYM_BINDING_NORMAL_TRIANGLE_TASK,
    AGENT_GYM_BINDING_NORMAL_SQUARE_TASK,
    AGENT_GYM_BINDING_SWAPPED_TRIANGLE_TASK,
    AGENT_GYM_BINDING_SWAPPED_SQUARE_TASK,
    AGENT_GYM_COMP_NTM_TASK,
    AGENT_GYM_COMP_NTF_TASK,
    AGENT_GYM_COMP_NSM_TASK,
    AGENT_GYM_COMP_NSF_TASK,
    AGENT_GYM_COMP_STM_TASK,
    AGENT_GYM_COMP_STF_TASK,
    AGENT_GYM_COMP_SSM_TASK,
    AGENT_GYM_COMP_SSF_TASK,
    AGENT_GYM_SEQ_NTMM_TASK,
    AGENT_GYM_SEQ_NTMF_TASK,
    AGENT_GYM_SEQ_NTFM_TASK,
    AGENT_GYM_SEQ_NTFF_TASK,
    AGENT_GYM_SEQ_NSMM_TASK,
    AGENT_GYM_SEQ_NSMF_TASK,
    AGENT_GYM_SEQ_NSFM_TASK,
    AGENT_GYM_SEQ_NSFF_TASK,
    AGENT_GYM_SEQ_STMM_TASK,
    AGENT_GYM_SEQ_STMF_TASK,
    AGENT_GYM_SEQ_STFM_TASK,
    AGENT_GYM_SEQ_STFF_TASK,
    AGENT_GYM_SEQ_SSMM_TASK,
    AGENT_GYM_SEQ_SSMF_TASK,
    AGENT_GYM_SEQ_SSFM_TASK,
    AGENT_GYM_SEQ_SSFF_TASK,
];

pub static BENCHMARK_SUITE_V1: BenchmarkSuiteSpec = BenchmarkSuiteSpec {
    id: BENCHMARK_SUITE_V1_ID,
    title: "Phi-Agent Gym Suite v1",
    version: 1,
    tasks: &BENCHMARK_SUITE_V1_TASKS,
};

pub static BENCHMARK_SUITE_V2: BenchmarkSuiteSpec = BenchmarkSuiteSpec {
    id: BENCHMARK_SUITE_V2_ID,
    title: "Phi-Agent Gym Suite v2",
    version: 2,
    tasks: &BENCHMARK_SUITE_V2_TASKS,
};

pub static BENCHMARK_SUITE_V3: BenchmarkSuiteSpec = BenchmarkSuiteSpec {
    id: BENCHMARK_SUITE_V3_ID,
    title: "Phi-Agent Gym Suite v3",
    version: 3,
    tasks: &BENCHMARK_SUITE_V3_TASKS,
};

pub static BENCHMARK_SUITE_V4: BenchmarkSuiteSpec = BenchmarkSuiteSpec {
    id: BENCHMARK_SUITE_V4_ID,
    title: "Phi-Agent Gym Suite v4",
    version: 4,
    tasks: &BENCHMARK_SUITE_V4_TASKS,
};

pub static BENCHMARK_SUITE_V5: BenchmarkSuiteSpec = BenchmarkSuiteSpec {
    id: BENCHMARK_SUITE_V5_ID,
    title: "Phi-Agent Gym Suite v5",
    version: 5,
    tasks: &BENCHMARK_SUITE_V5_TASKS,
};

pub static BENCHMARK_SUITE_V6: BenchmarkSuiteSpec = BenchmarkSuiteSpec {
    id: BENCHMARK_SUITE_V6_ID,
    title: "Phi-Agent Gym Suite v6",
    version: 6,
    tasks: &BENCHMARK_SUITE_V6_TASKS,
};

pub static BENCHMARK_SUITE_V7: BenchmarkSuiteSpec = BenchmarkSuiteSpec {
    id: BENCHMARK_SUITE_V7_ID,
    title: "Phi-Agent Gym Suite v7",
    version: 7,
    tasks: &BENCHMARK_SUITE_V7_TASKS,
};

pub static BENCHMARK_SUITE_V8: BenchmarkSuiteSpec = BenchmarkSuiteSpec {
    id: BENCHMARK_SUITE_V8_ID,
    title: "Phi-Agent Gym Suite v8",
    version: 8,
    tasks: &BENCHMARK_SUITE_V8_TASKS,
};

pub static BENCHMARK_SUITE_V9: BenchmarkSuiteSpec = BenchmarkSuiteSpec {
    id: BENCHMARK_SUITE_V9_ID,
    title: "Phi-Agent Gym Suite v9",
    version: 9,
    tasks: &BENCHMARK_SUITE_V9_TASKS,
};

pub static BENCHMARK_SUITE_V10: BenchmarkSuiteSpec = BenchmarkSuiteSpec {
    id: BENCHMARK_SUITE_V10_ID,
    title: "Phi-Agent Gym Suite v10",
    version: 10,
    tasks: &BENCHMARK_SUITE_V10_TASKS,
};

pub static BENCHMARK_SUITE_V11: BenchmarkSuiteSpec = BenchmarkSuiteSpec {
    id: BENCHMARK_SUITE_V11_ID,
    title: "Phi-Agent Gym Suite v11",
    version: 11,
    tasks: &BENCHMARK_SUITE_V11_TASKS,
};

pub static BENCHMARK_SUITES: [&BenchmarkSuiteSpec; 11] = [
    &BENCHMARK_SUITE_V1,
    &BENCHMARK_SUITE_V2,
    &BENCHMARK_SUITE_V3,
    &BENCHMARK_SUITE_V4,
    &BENCHMARK_SUITE_V5,
    &BENCHMARK_SUITE_V6,
    &BENCHMARK_SUITE_V7,
    &BENCHMARK_SUITE_V8,
    &BENCHMARK_SUITE_V9,
    &BENCHMARK_SUITE_V10,
    &BENCHMARK_SUITE_V11,
];

pub fn benchmark_suites() -> &'static [&'static BenchmarkSuiteSpec] {
    &BENCHMARK_SUITES
}

pub fn benchmark_suite_by_id(id: &str) -> Option<&'static BenchmarkSuiteSpec> {
    BENCHMARK_SUITES.iter().copied().find(|suite| suite.id == id)
}

pub fn benchmark_suite_v1_tasks() -> &'static [BenchmarkTaskSpec] {
    &BENCHMARK_SUITE_V1_TASKS
}

pub fn benchmark_suite_v2_tasks() -> &'static [BenchmarkTaskSpec] {
    &BENCHMARK_SUITE_V2_TASKS
}

pub fn benchmark_suite_v3_tasks() -> &'static [BenchmarkTaskSpec] {
    &BENCHMARK_SUITE_V3_TASKS
}

pub fn benchmark_suite_v4_tasks() -> &'static [BenchmarkTaskSpec] {
    &BENCHMARK_SUITE_V4_TASKS
}

pub fn benchmark_suite_v5_tasks() -> &'static [BenchmarkTaskSpec] {
    &BENCHMARK_SUITE_V5_TASKS
}

pub fn benchmark_suite_v6_tasks() -> &'static [BenchmarkTaskSpec] {
    &BENCHMARK_SUITE_V6_TASKS
}

pub fn benchmark_suite_v7_tasks() -> &'static [BenchmarkTaskSpec] {
    &BENCHMARK_SUITE_V7_TASKS
}

pub fn benchmark_suite_v8_tasks() -> &'static [BenchmarkTaskSpec] {
    &BENCHMARK_SUITE_V8_TASKS
}

pub fn benchmark_suite_v9_tasks() -> &'static [BenchmarkTaskSpec] {
    &BENCHMARK_SUITE_V9_TASKS
}

pub fn benchmark_suite_v10_tasks() -> &'static [BenchmarkTaskSpec] {
    &BENCHMARK_SUITE_V10_TASKS
}

pub fn benchmark_suite_v11_tasks() -> &'static [BenchmarkTaskSpec] {
    &BENCHMARK_SUITE_V11_TASKS
}

pub fn benchmark_suites_for_task(task_id: &str) -> Vec<&'static BenchmarkSuiteSpec> {
    BENCHMARK_SUITES
        .iter()
        .copied()
        .filter(|suite| suite.tasks.iter().any(|task| task.id == task_id))
        .collect()
}

pub fn benchmark_task_by_id(id: &str) -> Option<&'static BenchmarkTaskSpec> {
    BENCHMARK_TASKS.iter().find(|task| task.id == id)
}

pub fn benchmark_task_by_rom_sha256(rom_sha256: &str) -> Option<&'static BenchmarkTaskSpec> {
    BENCHMARK_TASKS
        .iter()
        .find(|task| task.rom_sha256 == rom_sha256)
}

fn dark(pixel: &[u8]) -> bool {
    if pixel.len() < 4 {
        return false;
    }
    let r = u16::from(pixel[0]);
    let g = u16::from(pixel[1]);
    let b = u16::from(pixel[2]);
    (r + g + b) / 3 < 112
}

pub fn locate_agent_gym_player(video: &FrameBuffer) -> Result<PixelPoint, String> {
    let width = usize::try_from(video.width).map_err(|_| "video width overflow")?;
    let height = usize::try_from(video.height).map_err(|_| "video height overflow")?;

    if width < AGENT_GYM_PLAYER_SIZE || height < AGENT_GYM_PLAYER_SIZE {
        return Err(format!(
            "unexpected gym framebuffer {}x{}",
            width, height
        ));
    }
    if video.rgba8.len() != width * height * 4 {
        return Err("gym framebuffer byte length mismatch".into());
    }

    let mut best: Option<(usize, usize, usize)> = None;
    for y in 0..=height - AGENT_GYM_PLAYER_SIZE {
        for x in 0..=width - AGENT_GYM_PLAYER_SIZE {
            let mut dark_count = 0usize;
            for py in y..y + AGENT_GYM_PLAYER_SIZE {
                let row = py * width * 4;
                for px in x..x + AGENT_GYM_PLAYER_SIZE {
                    let offset = row + px * 4;
                    if dark(&video.rgba8[offset..offset + 4]) {
                        dark_count += 1;
                    }
                }
            }

            if best.is_none_or(|(_, _, count)| dark_count > count) {
                best = Some((x, y, dark_count));
            }
        }
    }

    let (x, y, count) = best.ok_or_else(|| "no candidate player patch found".to_owned())?;
    if count < 52 {
        return Err(format!(
            "solid player patch not found: best 8x8 dark-pixel count was {count}"
        ));
    }

    Ok(PixelPoint {
        x: i32::try_from(x).map_err(|_| "player x overflow")?,
        y: i32::try_from(y).map_err(|_| "player y overflow")?,
    })
}

pub fn benchmark_task_distance(task: &BenchmarkTaskSpec, point: PixelPoint) -> i32 {
    (point.x - task.target.x).abs() + (point.y - task.target.y).abs()
}

pub fn agent_gym_distance(point: PixelPoint) -> i32 {
    benchmark_task_distance(&AGENT_GYM_TASK, point)
}

pub fn agent_gym_score_1000(initial_distance: i32, final_distance: i32) -> u16 {
    if initial_distance <= 0 {
        return 1000;
    }

    let progress = (initial_distance - final_distance).clamp(0, initial_distance);
    u16::try_from((i64::from(progress) * 1000) / i64::from(initial_distance)).unwrap_or(0)
}

pub fn benchmark_task_success(task: &BenchmarkTaskSpec, point: PixelPoint) -> bool {
    benchmark_task_distance(task, point) <= task.success_distance
}

pub fn agent_gym_success(point: PixelPoint) -> bool {
    benchmark_task_success(&AGENT_GYM_TASK, point)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentGymScore {
    pub player: PixelPoint,
    pub target: PixelPoint,
    pub initial_distance: i32,
    pub final_distance: i32,
    pub progress: i32,
    pub score_1000: u16,
    pub success: bool,
}

pub fn score_benchmark_task_frame(
    video: &FrameBuffer,
    task: &BenchmarkTaskSpec,
) -> Result<AgentGymScore, String> {
    let player = locate_agent_gym_player(video)?;
    let final_distance = benchmark_task_distance(task, player);
    Ok(AgentGymScore {
        player,
        target: task.target,
        initial_distance: task.initial_distance,
        final_distance,
        progress: task.initial_distance - final_distance,
        score_1000: agent_gym_score_1000(task.initial_distance, final_distance),
        success: benchmark_task_success(task, player),
    })
}

pub fn score_agent_gym_frame(video: &FrameBuffer) -> Result<AgentGymScore, String> {
    score_benchmark_task_frame(video, &AGENT_GYM_TASK)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_frame(player: PixelPoint) -> FrameBuffer {
        let width = 160usize;
        let height = 144usize;
        let mut rgba8 = vec![255u8; width * height * 4];

        for y in player.y as usize..player.y as usize + AGENT_GYM_PLAYER_SIZE {
            for x in player.x as usize..player.x as usize + AGENT_GYM_PLAYER_SIZE {
                let offset = (y * width + x) * 4;
                rgba8[offset..offset + 4].copy_from_slice(&[0, 0, 0, 255]);
            }
        }

        FrameBuffer {
            width: width as u32,
            height: height as u32,
            rgba8,
        }
    }

    #[test]
    fn suite_v1_contains_two_distinct_tasks() {
        assert_eq!(benchmark_suite_v1_tasks().len(), 2);
        assert_ne!(AGENT_GYM_TASK.id, AGENT_GYM_MIRROR_TASK.id);
        assert_ne!(AGENT_GYM_TASK.start, AGENT_GYM_MIRROR_TASK.start);
        assert_ne!(AGENT_GYM_TASK.target, AGENT_GYM_MIRROR_TASK.target);
    }

    #[test]
    fn suite_v2_preserves_v1_and_adds_wall_detour() {
        assert_eq!(benchmark_suite_v2_tasks().len(), 3);
        assert_eq!(benchmark_suite_v2_tasks()[0].id, AGENT_GYM_ID);
        assert_eq!(benchmark_suite_v2_tasks()[1].id, AGENT_GYM_MIRROR_ID);
        assert_eq!(benchmark_suite_v2_tasks()[2].id, AGENT_GYM_WALL_ID);
        assert_eq!(benchmark_suite_by_id(BENCHMARK_SUITE_V1_ID).unwrap().version, 1);
        assert_eq!(benchmark_suite_by_id(BENCHMARK_SUITE_V2_ID).unwrap().version, 2);
    }

    #[test]
    fn suite_v3_preserves_v2_and_adds_balanced_temporal_cues() {
        let tasks = benchmark_suite_v3_tasks();
        assert_eq!(tasks.len(), 5);
        assert_eq!(tasks[0].id, AGENT_GYM_ID);
        assert_eq!(tasks[1].id, AGENT_GYM_MIRROR_ID);
        assert_eq!(tasks[2].id, AGENT_GYM_WALL_ID);
        assert_eq!(tasks[3].id, AGENT_GYM_TEMPORAL_LEFT_ID);
        assert_eq!(tasks[4].id, AGENT_GYM_TEMPORAL_RIGHT_ID);
        assert_eq!(benchmark_suite_by_id(BENCHMARK_SUITE_V3_ID).unwrap().version, 3);
        assert_eq!(
            AGENT_GYM_TEMPORAL_LEFT_TASK.prompt,
            AGENT_GYM_TEMPORAL_RIGHT_TASK.prompt
        );
        assert_eq!(
            AGENT_GYM_TEMPORAL_LEFT_TASK.allowed_buttons,
            AGENT_GYM_TEMPORAL_RIGHT_TASK.allowed_buttons
        );
        assert_ne!(
            AGENT_GYM_TEMPORAL_LEFT_TASK.target,
            AGENT_GYM_TEMPORAL_RIGHT_TASK.target
        );
    }

    #[test]
    fn suite_v4_preserves_v3_and_adds_balanced_multi_room_relays() {
        let tasks = benchmark_suite_v4_tasks();
        assert_eq!(tasks.len(), 7);
        assert_eq!(tasks[..5], BENCHMARK_SUITE_V3_TASKS);
        assert_eq!(tasks[5].id, AGENT_GYM_RELAY_LEFT_ID);
        assert_eq!(tasks[6].id, AGENT_GYM_RELAY_RIGHT_ID);
        assert_eq!(benchmark_suite_by_id(BENCHMARK_SUITE_V4_ID).unwrap().version, 4);
        assert_eq!(AGENT_GYM_RELAY_LEFT_TASK.prompt, AGENT_GYM_RELAY_RIGHT_TASK.prompt);
        assert_eq!(AGENT_GYM_RELAY_LEFT_TASK.allowed_buttons, AGENT_GYM_RELAY_RIGHT_TASK.allowed_buttons);
        assert_ne!(AGENT_GYM_RELAY_LEFT_TASK.target, AGENT_GYM_RELAY_RIGHT_TASK.target);
        assert_eq!(AGENT_GYM_RELAY_LEFT_TASK.oracle.len(), 7);
        assert_eq!(AGENT_GYM_RELAY_RIGHT_TASK.oracle.len(), 7);
    }

    #[test]
    fn suite_v5_preserves_v4_and_adds_balanced_stateful_key_gates() {
        let tasks = benchmark_suite_v5_tasks();
        assert_eq!(tasks.len(), 9);
        assert_eq!(tasks[..7], BENCHMARK_SUITE_V4_TASKS);
        assert_eq!(tasks[7].id, AGENT_GYM_KEY_GATE_LEFT_ID);
        assert_eq!(tasks[8].id, AGENT_GYM_KEY_GATE_RIGHT_ID);
        assert_eq!(benchmark_suite_by_id(BENCHMARK_SUITE_V5_ID).unwrap().version, 5);
        assert_eq!(AGENT_GYM_KEY_GATE_LEFT_TASK.prompt, AGENT_GYM_KEY_GATE_RIGHT_TASK.prompt);
        assert_eq!(AGENT_GYM_KEY_GATE_LEFT_TASK.allowed_buttons, AGENT_GYM_KEY_GATE_RIGHT_TASK.allowed_buttons);
        assert_eq!(AGENT_GYM_KEY_GATE_LEFT_TASK.target, AGENT_GYM_KEY_GATE_RIGHT_TASK.target);
        assert_eq!(AGENT_GYM_KEY_GATE_LEFT_TASK.oracle.len(), 6);
        assert_eq!(AGENT_GYM_KEY_GATE_RIGHT_TASK.oracle.len(), 6);
    }

    #[test]
    fn suite_v6_preserves_v5_and_adds_balanced_ordered_power_chains() {
        let tasks = benchmark_suite_v6_tasks();
        assert_eq!(tasks.len(), 11);
        assert_eq!(tasks[..9], BENCHMARK_SUITE_V5_TASKS);
        assert_eq!(tasks[9].id, AGENT_GYM_POWER_CHAIN_LEFT_ID);
        assert_eq!(tasks[10].id, AGENT_GYM_POWER_CHAIN_RIGHT_ID);
        assert_eq!(benchmark_suite_by_id(BENCHMARK_SUITE_V6_ID).unwrap().version, 6);
        assert_eq!(AGENT_GYM_POWER_CHAIN_LEFT_TASK.prompt, AGENT_GYM_POWER_CHAIN_RIGHT_TASK.prompt);
        assert_eq!(AGENT_GYM_POWER_CHAIN_LEFT_TASK.target, AGENT_GYM_POWER_CHAIN_RIGHT_TASK.target);
        assert_eq!(AGENT_GYM_POWER_CHAIN_LEFT_TASK.oracle.len(), 10);
        assert_eq!(AGENT_GYM_POWER_CHAIN_RIGHT_TASK.oracle.len(), 10);
    }

    #[test]
    fn suite_v7_preserves_v6_and_adds_balanced_conditional_branches() {
        let tasks = benchmark_suite_v7_tasks();
        assert_eq!(tasks.len(), 13);
        assert_eq!(tasks[..11], BENCHMARK_SUITE_V6_TASKS);
        assert_eq!(tasks[11].id, AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_ID);
        assert_eq!(tasks[12].id, AGENT_GYM_BRANCH_SELECTOR_SQUARE_ID);
        assert_eq!(benchmark_suite_by_id(BENCHMARK_SUITE_V7_ID).unwrap().version, 7);
        assert_eq!(
            AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_TASK.prompt,
            AGENT_GYM_BRANCH_SELECTOR_SQUARE_TASK.prompt
        );
        assert_eq!(
            AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_TASK.target,
            AGENT_GYM_BRANCH_SELECTOR_SQUARE_TASK.target
        );
        assert_eq!(AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_TASK.oracle.len(), 10);
        assert_eq!(AGENT_GYM_BRANCH_SELECTOR_SQUARE_TASK.oracle.len(), 10);
    }

    #[test]
    fn suite_v8_preserves_v7_and_adds_factorial_nested_branches() {
        let tasks = benchmark_suite_v8_tasks();
        assert_eq!(tasks.len(), 17);
        assert_eq!(tasks[..13], BENCHMARK_SUITE_V7_TASKS);
        assert_eq!(tasks[13].id, AGENT_GYM_NESTED_TRIANGLE_CIRCLE_ID);
        assert_eq!(tasks[14].id, AGENT_GYM_NESTED_TRIANGLE_CROSS_ID);
        assert_eq!(tasks[15].id, AGENT_GYM_NESTED_SQUARE_CIRCLE_ID);
        assert_eq!(tasks[16].id, AGENT_GYM_NESTED_SQUARE_CROSS_ID);
        assert_eq!(benchmark_suite_by_id(BENCHMARK_SUITE_V8_ID).unwrap().version, 8);
        assert_eq!(
            AGENT_GYM_NESTED_TRIANGLE_CIRCLE_TASK.prompt,
            AGENT_GYM_NESTED_SQUARE_CROSS_TASK.prompt
        );
        assert_eq!(AGENT_GYM_NESTED_TRIANGLE_CIRCLE_TASK.oracle.len(), 16);
        assert_eq!(AGENT_GYM_NESTED_TRIANGLE_CROSS_TASK.oracle.len(), 16);
        assert_eq!(AGENT_GYM_NESTED_SQUARE_CIRCLE_TASK.oracle.len(), 16);
        assert_eq!(AGENT_GYM_NESTED_SQUARE_CROSS_TASK.oracle.len(), 16);
    }

    #[test]
    fn suite_v9_preserves_v8_and_adds_relational_binding_memory_factorial() {
        let tasks = benchmark_suite_v9_tasks();
        assert_eq!(tasks.len(), 21);
        assert_eq!(tasks[..17], BENCHMARK_SUITE_V8_TASKS);
        assert_eq!(tasks[17].id, AGENT_GYM_BINDING_NORMAL_TRIANGLE_ID);
        assert_eq!(tasks[18].id, AGENT_GYM_BINDING_NORMAL_SQUARE_ID);
        assert_eq!(tasks[19].id, AGENT_GYM_BINDING_SWAPPED_TRIANGLE_ID);
        assert_eq!(tasks[20].id, AGENT_GYM_BINDING_SWAPPED_SQUARE_ID);
        assert_eq!(benchmark_suite_by_id(BENCHMARK_SUITE_V9_ID).unwrap().version, 9);
        let binding_tasks = [
            AGENT_GYM_BINDING_NORMAL_TRIANGLE_TASK,
            AGENT_GYM_BINDING_NORMAL_SQUARE_TASK,
            AGENT_GYM_BINDING_SWAPPED_TRIANGLE_TASK,
            AGENT_GYM_BINDING_SWAPPED_SQUARE_TASK,
        ];
        for task in &binding_tasks[1..] {
            assert_eq!(task.prompt, binding_tasks[0].prompt);
            assert_eq!(task.allowed_buttons, binding_tasks[0].allowed_buttons);
        }
        let prompt = binding_tasks[0].prompt.to_ascii_lowercase();
        assert!(!prompt.contains("normal"));
        assert!(!prompt.contains("swapped"));
        assert!(!prompt.contains("triangle is left"));
        assert!(!prompt.contains("triangle is right"));
        assert!(!prompt.contains("square is left"));
        assert!(!prompt.contains("square is right"));
        assert_eq!(AGENT_GYM_BINDING_NORMAL_TRIANGLE_TASK.oracle.len(), 4);
        assert_eq!(AGENT_GYM_BINDING_NORMAL_SQUARE_TASK.oracle.len(), 4);
        assert_eq!(AGENT_GYM_BINDING_SWAPPED_TRIANGLE_TASK.oracle.len(), 4);
        assert_eq!(AGENT_GYM_BINDING_SWAPPED_SQUARE_TASK.oracle.len(), 4);
    }

    #[test]
    fn suite_v10_preserves_v9_and_adds_compositional_recall_factorial() {
        let tasks = benchmark_suite_v10_tasks();
        assert_eq!(tasks.len(), 29);
        assert_eq!(tasks[..21], BENCHMARK_SUITE_V9_TASKS);
        let ids = [
            AGENT_GYM_COMP_NTM_ID, AGENT_GYM_COMP_NTF_ID,
            AGENT_GYM_COMP_NSM_ID, AGENT_GYM_COMP_NSF_ID,
            AGENT_GYM_COMP_STM_ID, AGENT_GYM_COMP_STF_ID,
            AGENT_GYM_COMP_SSM_ID, AGENT_GYM_COMP_SSF_ID,
        ];
        for (offset, id) in ids.iter().enumerate() {
            assert_eq!(tasks[21 + offset].id, *id);
        }
        assert_eq!(benchmark_suite_by_id(BENCHMARK_SUITE_V10_ID).unwrap().version, 10);

        let comp = [
            AGENT_GYM_COMP_NTM_TASK, AGENT_GYM_COMP_NTF_TASK,
            AGENT_GYM_COMP_NSM_TASK, AGENT_GYM_COMP_NSF_TASK,
            AGENT_GYM_COMP_STM_TASK, AGENT_GYM_COMP_STF_TASK,
            AGENT_GYM_COMP_SSM_TASK, AGENT_GYM_COMP_SSF_TASK,
        ];
        for task in &comp[1..] {
            assert_eq!(task.prompt, comp[0].prompt);
            assert_eq!(task.allowed_buttons, comp[0].allowed_buttons);
        }
        let prompt = comp[0].prompt.to_ascii_lowercase();
        for leak in [
            "normal /", "swapped /", "normal-triangle", "swapped-triangle",
            "normal-square", "swapped-square", "correct door is", "answer is left",
            "answer is right",
        ] {
            assert!(!prompt.contains(leak), "provider prompt leaked variant detail: {leak}");
        }
        assert!(prompt.contains("match (=)"));
        assert!(prompt.contains("flip (x)"));
        assert!(comp.iter().all(|task| task.oracle.len() == 4));
    }

    #[test]
    fn suite_v11_preserves_v10_and_adds_sequential_rule_factorial() {
        let tasks = benchmark_suite_v11_tasks();
        assert_eq!(tasks.len(), 45);
        assert_eq!(tasks[..29], BENCHMARK_SUITE_V10_TASKS);
        let ids = [AGENT_GYM_SEQ_NTMM_ID, AGENT_GYM_SEQ_NTMF_ID, AGENT_GYM_SEQ_NTFM_ID, AGENT_GYM_SEQ_NTFF_ID, AGENT_GYM_SEQ_NSMM_ID, AGENT_GYM_SEQ_NSMF_ID, AGENT_GYM_SEQ_NSFM_ID, AGENT_GYM_SEQ_NSFF_ID, AGENT_GYM_SEQ_STMM_ID, AGENT_GYM_SEQ_STMF_ID, AGENT_GYM_SEQ_STFM_ID, AGENT_GYM_SEQ_STFF_ID, AGENT_GYM_SEQ_SSMM_ID, AGENT_GYM_SEQ_SSMF_ID, AGENT_GYM_SEQ_SSFM_ID, AGENT_GYM_SEQ_SSFF_ID];
        for (offset, id) in ids.iter().enumerate() {
            assert_eq!(tasks[29 + offset].id, *id);
        }
        assert_eq!(benchmark_suite_by_id(BENCHMARK_SUITE_V11_ID).unwrap().version, 11);

        let seq = [AGENT_GYM_SEQ_NTMM_TASK, AGENT_GYM_SEQ_NTMF_TASK, AGENT_GYM_SEQ_NTFM_TASK, AGENT_GYM_SEQ_NTFF_TASK, AGENT_GYM_SEQ_NSMM_TASK, AGENT_GYM_SEQ_NSMF_TASK, AGENT_GYM_SEQ_NSFM_TASK, AGENT_GYM_SEQ_NSFF_TASK, AGENT_GYM_SEQ_STMM_TASK, AGENT_GYM_SEQ_STMF_TASK, AGENT_GYM_SEQ_STFM_TASK, AGENT_GYM_SEQ_STFF_TASK, AGENT_GYM_SEQ_SSMM_TASK, AGENT_GYM_SEQ_SSMF_TASK, AGENT_GYM_SEQ_SSFM_TASK, AGENT_GYM_SEQ_SSFF_TASK];
        for task in &seq[1..] {
            assert_eq!(task.prompt, seq[0].prompt);
            assert_eq!(task.allowed_buttons, seq[0].allowed_buttons);
        }
        let prompt = seq[0].prompt.to_ascii_lowercase();
        for leak in [
            "normal /", "swapped /", "normal-triangle", "swapped-triangle",
            "normal-square", "swapped-square", "correct door is", "answer is left",
            "answer is right",
        ] {
            assert!(!prompt.contains(leak), "provider prompt leaked variant detail: {leak}");
        }
        assert!(prompt.contains("operator 1"));
        assert!(prompt.contains("operator 2"));
        assert!(prompt.contains("intermediate"));
        assert!(seq.iter().all(|task| task.oracle.len() == 6));
    }

    #[test]
    fn suite_membership_is_separate_from_task_origin() {
        let memberships = benchmark_suites_for_task(AGENT_GYM_ID);
        assert_eq!(memberships.len(), 11);
        assert_eq!(memberships[0].id, BENCHMARK_SUITE_V1_ID);
        assert_eq!(memberships[1].id, BENCHMARK_SUITE_V2_ID);
        assert_eq!(memberships[2].id, BENCHMARK_SUITE_V3_ID);
        assert_eq!(memberships[3].id, BENCHMARK_SUITE_V4_ID);
        assert_eq!(memberships[4].id, BENCHMARK_SUITE_V5_ID);
        assert_eq!(memberships[5].id, BENCHMARK_SUITE_V6_ID);
        assert_eq!(memberships[6].id, BENCHMARK_SUITE_V7_ID);
        assert_eq!(memberships[7].id, BENCHMARK_SUITE_V8_ID);
        assert_eq!(memberships[8].id, BENCHMARK_SUITE_V9_ID);
        assert_eq!(memberships[9].id, BENCHMARK_SUITE_V10_ID);
        assert_eq!(memberships[10].id, BENCHMARK_SUITE_V11_ID);

        let wall_memberships = benchmark_suites_for_task(AGENT_GYM_WALL_ID);
        assert_eq!(wall_memberships.len(), 10);
        assert_eq!(wall_memberships[0].id, BENCHMARK_SUITE_V2_ID);
        assert_eq!(wall_memberships[1].id, BENCHMARK_SUITE_V3_ID);
        assert_eq!(wall_memberships[2].id, BENCHMARK_SUITE_V4_ID);
        assert_eq!(wall_memberships[3].id, BENCHMARK_SUITE_V5_ID);
        assert_eq!(wall_memberships[4].id, BENCHMARK_SUITE_V6_ID);
        assert_eq!(wall_memberships[5].id, BENCHMARK_SUITE_V7_ID);
        assert_eq!(wall_memberships[6].id, BENCHMARK_SUITE_V8_ID);
        assert_eq!(wall_memberships[7].id, BENCHMARK_SUITE_V9_ID);
        assert_eq!(wall_memberships[8].id, BENCHMARK_SUITE_V10_ID);
        assert_eq!(wall_memberships[9].id, BENCHMARK_SUITE_V11_ID);

        let temporal_memberships = benchmark_suites_for_task(AGENT_GYM_TEMPORAL_LEFT_ID);
        assert_eq!(temporal_memberships.len(), 9);
        assert_eq!(temporal_memberships[0].id, BENCHMARK_SUITE_V3_ID);
        assert_eq!(temporal_memberships[1].id, BENCHMARK_SUITE_V4_ID);
        assert_eq!(temporal_memberships[2].id, BENCHMARK_SUITE_V5_ID);
        assert_eq!(temporal_memberships[3].id, BENCHMARK_SUITE_V6_ID);
        assert_eq!(temporal_memberships[4].id, BENCHMARK_SUITE_V7_ID);
        assert_eq!(temporal_memberships[5].id, BENCHMARK_SUITE_V8_ID);
        assert_eq!(temporal_memberships[6].id, BENCHMARK_SUITE_V9_ID);
        assert_eq!(temporal_memberships[7].id, BENCHMARK_SUITE_V10_ID);
        assert_eq!(temporal_memberships[8].id, BENCHMARK_SUITE_V11_ID);

        let relay_memberships = benchmark_suites_for_task(AGENT_GYM_RELAY_LEFT_ID);
        assert_eq!(relay_memberships.len(), 8);
        assert_eq!(relay_memberships[0].id, BENCHMARK_SUITE_V4_ID);
        assert_eq!(relay_memberships[1].id, BENCHMARK_SUITE_V5_ID);
        assert_eq!(relay_memberships[2].id, BENCHMARK_SUITE_V6_ID);
        assert_eq!(relay_memberships[3].id, BENCHMARK_SUITE_V7_ID);
        assert_eq!(relay_memberships[4].id, BENCHMARK_SUITE_V8_ID);
        assert_eq!(relay_memberships[5].id, BENCHMARK_SUITE_V9_ID);
        assert_eq!(relay_memberships[6].id, BENCHMARK_SUITE_V10_ID);
        assert_eq!(relay_memberships[7].id, BENCHMARK_SUITE_V11_ID);

        let key_gate_memberships = benchmark_suites_for_task(AGENT_GYM_KEY_GATE_LEFT_ID);
        assert_eq!(key_gate_memberships.len(), 7);
        assert_eq!(key_gate_memberships[0].id, BENCHMARK_SUITE_V5_ID);
        assert_eq!(key_gate_memberships[1].id, BENCHMARK_SUITE_V6_ID);
        assert_eq!(key_gate_memberships[2].id, BENCHMARK_SUITE_V7_ID);
        assert_eq!(key_gate_memberships[3].id, BENCHMARK_SUITE_V8_ID);
        assert_eq!(key_gate_memberships[4].id, BENCHMARK_SUITE_V9_ID);
        assert_eq!(key_gate_memberships[5].id, BENCHMARK_SUITE_V10_ID);
        assert_eq!(key_gate_memberships[6].id, BENCHMARK_SUITE_V11_ID);

        let power_memberships = benchmark_suites_for_task(AGENT_GYM_POWER_CHAIN_LEFT_ID);
        assert_eq!(power_memberships.len(), 6);
        assert_eq!(power_memberships[0].id, BENCHMARK_SUITE_V6_ID);
        assert_eq!(power_memberships[1].id, BENCHMARK_SUITE_V7_ID);
        assert_eq!(power_memberships[2].id, BENCHMARK_SUITE_V8_ID);
        assert_eq!(power_memberships[3].id, BENCHMARK_SUITE_V9_ID);
        assert_eq!(power_memberships[4].id, BENCHMARK_SUITE_V10_ID);
        assert_eq!(power_memberships[5].id, BENCHMARK_SUITE_V11_ID);

        let branch_memberships =
            benchmark_suites_for_task(AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_ID);
        assert_eq!(branch_memberships.len(), 5);
        assert_eq!(branch_memberships[0].id, BENCHMARK_SUITE_V7_ID);
        assert_eq!(branch_memberships[1].id, BENCHMARK_SUITE_V8_ID);
        assert_eq!(branch_memberships[2].id, BENCHMARK_SUITE_V9_ID);
        assert_eq!(branch_memberships[3].id, BENCHMARK_SUITE_V10_ID);
        assert_eq!(branch_memberships[4].id, BENCHMARK_SUITE_V11_ID);

        let nested_memberships =
            benchmark_suites_for_task(AGENT_GYM_NESTED_TRIANGLE_CIRCLE_ID);
        assert_eq!(nested_memberships.len(), 4);
        assert_eq!(nested_memberships[0].id, BENCHMARK_SUITE_V8_ID);
        assert_eq!(nested_memberships[1].id, BENCHMARK_SUITE_V9_ID);
        assert_eq!(nested_memberships[2].id, BENCHMARK_SUITE_V10_ID);
        assert_eq!(nested_memberships[3].id, BENCHMARK_SUITE_V11_ID);
    }

    #[test]
    fn registry_resolves_original_task_by_id_and_hash() {
        assert_eq!(
            benchmark_task_by_id(AGENT_GYM_ID).map(|task| task.id),
            Some(AGENT_GYM_ID)
        );
        assert_eq!(
            benchmark_task_by_rom_sha256(AGENT_GYM_ROM_SHA256).map(|task| task.id),
            Some(AGENT_GYM_ID)
        );
    }

    #[test]
    fn registry_resolves_wall_task_by_id() {
        assert_eq!(
            benchmark_task_by_id(AGENT_GYM_WALL_ID).map(|task| task.id),
            Some(AGENT_GYM_WALL_ID)
        );
        assert_eq!(AGENT_GYM_WALL_SOURCE_SHA256.len(), 64);
        assert_eq!(AGENT_GYM_WALL_ROM_SHA256.len(), 64);
        assert_eq!(AGENT_GYM_WALL_TASK.oracle.len(), 3);
    }

    #[test]
    fn registry_resolves_temporal_tasks_by_id() {
        assert_eq!(
            benchmark_task_by_id(AGENT_GYM_TEMPORAL_LEFT_ID).map(|task| task.id),
            Some(AGENT_GYM_TEMPORAL_LEFT_ID)
        );
        assert_eq!(
            benchmark_task_by_id(AGENT_GYM_TEMPORAL_RIGHT_ID).map(|task| task.id),
            Some(AGENT_GYM_TEMPORAL_RIGHT_ID)
        );
        assert_eq!(AGENT_GYM_TEMPORAL_LEFT_TASK.oracle[0].button, "A");
        assert_eq!(AGENT_GYM_TEMPORAL_LEFT_TASK.oracle[1].button, "WAIT");
        assert_eq!(AGENT_GYM_TEMPORAL_RIGHT_TASK.oracle[2].button, "RIGHT");
    }

    #[test]
    fn registry_resolves_relay_tasks_by_id() {
        assert_eq!(
            benchmark_task_by_id(AGENT_GYM_RELAY_LEFT_ID).map(|task| task.id),
            Some(AGENT_GYM_RELAY_LEFT_ID)
        );
        assert_eq!(
            benchmark_task_by_id(AGENT_GYM_RELAY_RIGHT_ID).map(|task| task.id),
            Some(AGENT_GYM_RELAY_RIGHT_ID)
        );
        assert_eq!(AGENT_GYM_RELAY_LEFT_TASK.oracle[0].button, "A");
        assert_eq!(AGENT_GYM_RELAY_LEFT_TASK.oracle[2].button, "DOWN");
        assert_eq!(AGENT_GYM_RELAY_LEFT_TASK.oracle[6].button, "LEFT");
        assert_eq!(AGENT_GYM_RELAY_RIGHT_TASK.oracle[6].button, "RIGHT");
    }

    #[test]
    fn registry_resolves_key_gate_tasks_by_id() {
        assert_eq!(
            benchmark_task_by_id(AGENT_GYM_KEY_GATE_LEFT_ID).map(|task| task.id),
            Some(AGENT_GYM_KEY_GATE_LEFT_ID)
        );
        assert_eq!(
            benchmark_task_by_id(AGENT_GYM_KEY_GATE_RIGHT_ID).map(|task| task.id),
            Some(AGENT_GYM_KEY_GATE_RIGHT_ID)
        );
        assert_eq!(AGENT_GYM_KEY_GATE_LEFT_TASK.oracle[1].button, "A");
        assert_eq!(AGENT_GYM_KEY_GATE_LEFT_TASK.oracle[4].button, "A");
        assert_eq!(AGENT_GYM_KEY_GATE_RIGHT_TASK.oracle[0].button, "RIGHT");
    }

    #[test]
    fn registry_resolves_power_chain_tasks_by_id() {
        assert_eq!(
            benchmark_task_by_id(AGENT_GYM_POWER_CHAIN_LEFT_ID).map(|task| task.id),
            Some(AGENT_GYM_POWER_CHAIN_LEFT_ID)
        );
        assert_eq!(
            benchmark_task_by_id(AGENT_GYM_POWER_CHAIN_RIGHT_ID).map(|task| task.id),
            Some(AGENT_GYM_POWER_CHAIN_RIGHT_ID)
        );
        assert_eq!(AGENT_GYM_POWER_CHAIN_LEFT_TASK.oracle[1].button, "A");
        assert_eq!(AGENT_GYM_POWER_CHAIN_LEFT_TASK.oracle[3].button, "WAIT");
        assert_eq!(AGENT_GYM_POWER_CHAIN_LEFT_TASK.oracle[4].button, "A");
        assert_eq!(AGENT_GYM_POWER_CHAIN_LEFT_TASK.oracle[7].button, "A");
    }

    #[test]
    fn registry_resolves_branch_selector_tasks_by_id() {
        assert_eq!(
            benchmark_task_by_id(AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_ID).map(|task| task.id),
            Some(AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_ID)
        );
        assert_eq!(
            benchmark_task_by_id(AGENT_GYM_BRANCH_SELECTOR_SQUARE_ID).map(|task| task.id),
            Some(AGENT_GYM_BRANCH_SELECTOR_SQUARE_ID)
        );
        assert_eq!(AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_TASK.oracle[0].button, "LEFT");
        assert_eq!(AGENT_GYM_BRANCH_SELECTOR_SQUARE_TASK.oracle[0].button, "RIGHT");
        assert_eq!(AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_TASK.oracle[1].button, "A");
        assert_eq!(AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_TASK.oracle[4].button, "A");
        assert_eq!(AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_TASK.oracle[7].button, "A");
    }

    #[test]
    fn registry_resolves_nested_branch_tasks_by_id() {
        for id in [
            AGENT_GYM_NESTED_TRIANGLE_CIRCLE_ID,
            AGENT_GYM_NESTED_TRIANGLE_CROSS_ID,
            AGENT_GYM_NESTED_SQUARE_CIRCLE_ID,
            AGENT_GYM_NESTED_SQUARE_CROSS_ID,
        ] {
            assert_eq!(benchmark_task_by_id(id).map(|task| task.id), Some(id));
        }
        assert_eq!(AGENT_GYM_NESTED_TRIANGLE_CIRCLE_TASK.oracle[0].button, "LEFT");
        assert_eq!(AGENT_GYM_NESTED_TRIANGLE_CIRCLE_TASK.oracle[5].button, "LEFT");
        assert_eq!(AGENT_GYM_NESTED_TRIANGLE_CROSS_TASK.oracle[5].button, "RIGHT");
        assert_eq!(AGENT_GYM_NESTED_SQUARE_CIRCLE_TASK.oracle[0].button, "RIGHT");
        assert_eq!(AGENT_GYM_NESTED_SQUARE_CROSS_TASK.oracle[5].button, "RIGHT");
    }

    #[test]
    fn registry_resolves_binding_memory_tasks_by_id() {
        for id in [
            AGENT_GYM_BINDING_NORMAL_TRIANGLE_ID,
            AGENT_GYM_BINDING_NORMAL_SQUARE_ID,
            AGENT_GYM_BINDING_SWAPPED_TRIANGLE_ID,
            AGENT_GYM_BINDING_SWAPPED_SQUARE_ID,
        ] {
            assert_eq!(benchmark_task_by_id(id).map(|task| task.id), Some(id));
            let memberships = benchmark_suites_for_task(id);
            assert_eq!(memberships.len(), 3);
            assert_eq!(memberships[0].id, BENCHMARK_SUITE_V9_ID);
            assert_eq!(memberships[1].id, BENCHMARK_SUITE_V10_ID);
            assert_eq!(memberships[2].id, BENCHMARK_SUITE_V11_ID);
        }
        assert_eq!(AGENT_GYM_BINDING_NORMAL_TRIANGLE_TASK.target, AGENT_GYM_BINDING_LEFT_TARGET);
        assert_eq!(AGENT_GYM_BINDING_NORMAL_SQUARE_TASK.target, AGENT_GYM_BINDING_RIGHT_TARGET);
        assert_eq!(AGENT_GYM_BINDING_SWAPPED_TRIANGLE_TASK.target, AGENT_GYM_BINDING_RIGHT_TARGET);
        assert_eq!(AGENT_GYM_BINDING_SWAPPED_SQUARE_TASK.target, AGENT_GYM_BINDING_LEFT_TARGET);
    }

    #[test]
    fn registry_resolves_compositional_recall_tasks_by_id() {
        let tasks = [
            AGENT_GYM_COMP_NTM_TASK, AGENT_GYM_COMP_NTF_TASK,
            AGENT_GYM_COMP_NSM_TASK, AGENT_GYM_COMP_NSF_TASK,
            AGENT_GYM_COMP_STM_TASK, AGENT_GYM_COMP_STF_TASK,
            AGENT_GYM_COMP_SSM_TASK, AGENT_GYM_COMP_SSF_TASK,
        ];
        for task in tasks {
            assert_eq!(benchmark_task_by_id(task.id).map(|found| found.id), Some(task.id));
            assert_eq!(benchmark_task_by_rom_sha256(task.rom_sha256).map(|found| found.id), Some(task.id));
            let memberships = benchmark_suites_for_task(task.id);
            assert_eq!(memberships.len(), 2);
            assert_eq!(memberships[0].id, BENCHMARK_SUITE_V10_ID);
            assert_eq!(memberships[1].id, BENCHMARK_SUITE_V11_ID);
        }
        assert_eq!(AGENT_GYM_COMP_NTM_TASK.target, AGENT_GYM_BINDING_LEFT_TARGET);
        assert_eq!(AGENT_GYM_COMP_NTF_TASK.target, AGENT_GYM_BINDING_RIGHT_TARGET);
        assert_eq!(AGENT_GYM_COMP_NSM_TASK.target, AGENT_GYM_BINDING_RIGHT_TARGET);
        assert_eq!(AGENT_GYM_COMP_NSF_TASK.target, AGENT_GYM_BINDING_LEFT_TARGET);
        assert_eq!(AGENT_GYM_COMP_STM_TASK.target, AGENT_GYM_BINDING_RIGHT_TARGET);
        assert_eq!(AGENT_GYM_COMP_STF_TASK.target, AGENT_GYM_BINDING_LEFT_TARGET);
        assert_eq!(AGENT_GYM_COMP_SSM_TASK.target, AGENT_GYM_BINDING_LEFT_TARGET);
        assert_eq!(AGENT_GYM_COMP_SSF_TASK.target, AGENT_GYM_BINDING_RIGHT_TARGET);
    }

    #[test]
    fn registry_resolves_sequential_rule_tasks_by_id_and_hash() {
        let tasks = [AGENT_GYM_SEQ_NTMM_TASK, AGENT_GYM_SEQ_NTMF_TASK, AGENT_GYM_SEQ_NTFM_TASK, AGENT_GYM_SEQ_NTFF_TASK, AGENT_GYM_SEQ_NSMM_TASK, AGENT_GYM_SEQ_NSMF_TASK, AGENT_GYM_SEQ_NSFM_TASK, AGENT_GYM_SEQ_NSFF_TASK, AGENT_GYM_SEQ_STMM_TASK, AGENT_GYM_SEQ_STMF_TASK, AGENT_GYM_SEQ_STFM_TASK, AGENT_GYM_SEQ_STFF_TASK, AGENT_GYM_SEQ_SSMM_TASK, AGENT_GYM_SEQ_SSMF_TASK, AGENT_GYM_SEQ_SSFM_TASK, AGENT_GYM_SEQ_SSFF_TASK];
        for task in tasks {
            assert_eq!(benchmark_task_by_id(task.id).map(|found| found.id), Some(task.id));
            assert_eq!(benchmark_task_by_rom_sha256(task.rom_sha256).map(|found| found.id), Some(task.id));
            let memberships = benchmark_suites_for_task(task.id);
            assert_eq!(memberships.len(), 1);
            assert_eq!(memberships[0].id, BENCHMARK_SUITE_V11_ID);
            assert_eq!(task.rom_sha256.len(), 64);
            assert_eq!(task.source_sha256.len(), 64);
        }
    }

    #[test]
    fn registry_resolves_mirror_task_by_exact_hash() {
        assert_eq!(
            benchmark_task_by_id(AGENT_GYM_MIRROR_ID).map(|task| task.id),
            Some(AGENT_GYM_MIRROR_ID)
        );
        assert_eq!(
            benchmark_task_by_rom_sha256(AGENT_GYM_MIRROR_ROM_SHA256).map(|task| task.id),
            Some(AGENT_GYM_MIRROR_ID)
        );
        assert_eq!(AGENT_GYM_MIRROR_ROM_SHA256.len(), 64);
        assert_eq!(AGENT_GYM_MIRROR_SOURCE_SHA256.len(), 64);
    }

    #[test]
    fn both_task_geometries_have_same_frozen_distance() {
        assert_eq!(
            benchmark_task_distance(&AGENT_GYM_TASK, AGENT_GYM_TASK.start),
            AGENT_GYM_INITIAL_DISTANCE
        );
        assert_eq!(
            benchmark_task_distance(&AGENT_GYM_MIRROR_TASK, AGENT_GYM_MIRROR_TASK.start),
            AGENT_GYM_INITIAL_DISTANCE
        );
    }

    #[test]
    fn locates_solid_player_from_pixels() {
        let point = PixelPoint { x: 33, y: 44 };
        assert_eq!(
            locate_agent_gym_player(&synthetic_frame(point)).expect("player"),
            point
        );
    }

    #[test]
    fn original_compatibility_wrappers_match_task_registry() {
        assert_eq!(agent_gym_distance(AGENT_GYM_START), AGENT_GYM_INITIAL_DISTANCE);
        assert_eq!(agent_gym_distance(AGENT_GYM_TARGET), 0);
        assert_eq!(agent_gym_score_1000(AGENT_GYM_INITIAL_DISTANCE, 0), 1000);
        assert!(agent_gym_success(AGENT_GYM_TARGET));
    }

    #[test]
    fn scores_registered_targets_at_full_credit() {
        for task in BENCHMARK_TASKS.iter() {
            let score = score_benchmark_task_frame(&synthetic_frame(task.target), task)
                .expect("score target");
            assert_eq!(score.final_distance, 0);
            assert_eq!(score.progress, task.initial_distance);
            assert_eq!(score.score_1000, 1000);
            assert!(score.success);
        }
    }

    #[test]
    fn scores_registered_starts_at_zero_progress() {
        for task in BENCHMARK_TASKS.iter() {
            let score = score_benchmark_task_frame(&synthetic_frame(task.start), task)
                .expect("score start");
            assert_eq!(score.final_distance, task.initial_distance);
            assert_eq!(score.progress, 0);
            assert_eq!(score.score_1000, 0);
            assert!(!score.success);
        }
    }
}
