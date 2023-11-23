use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct NavInfoPrivate {
    #[serde(flatten)]
    inner: Option<NavInfo>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NavInfo {
    pub email_verified: i64,
    pub face: String,
    pub face_nft: i64,
    pub face_nft_type: i64,
    pub level_info: LevelInfo,
    pub mid: i64,
    pub mobile_verified: i64,
    pub money: f64,
    pub moral: i64,
    pub official: Official,
    #[serde(rename = "officialVerify")]
    pub official_verify: OfficialVerify,
    pub pendant: Pendant,
    pub scores: i64,
    pub uname: String,
    #[serde(rename = "vipDueDate")]
    pub vip_due_date: i64,
    #[serde(rename = "vipStatus")]
    pub vip_status: i64,
    #[serde(rename = "vipType")]
    pub vip_type: i64,
    pub vip_pay_type: i64,
    pub vip_theme_type: i64,
    pub vip_label: VipLabel,
    pub vip_avatar_subscript: i64,
    pub vip_nickname_color: String,
    pub vip: Vip,
    pub wallet: Wallet,
    pub has_shop: bool,
    pub shop_url: String,
    pub allowance_count: i64,
    pub answer_status: i64,
    pub is_senior_member: i64,
    pub is_jury: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelInfo {
    pub current_level: i64,
    pub current_min: i64,
    pub current_exp: i64,
    pub next_exp: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Official {
    pub role: i64,
    pub title: String,
    pub desc: String,
    #[serde(rename = "type")]
    pub type_field: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfficialVerify {
    #[serde(rename = "type")]
    pub type_field: i64,
    pub desc: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pendant {
    pub pid: i64,
    pub name: String,
    pub image: String,
    pub expire: i64,
    pub image_enhance: String,
    pub image_enhance_frame: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VipLabel {
    pub path: String,
    pub text: String,
    pub label_theme: String,
    pub text_color: String,
    pub bg_style: i64,
    pub bg_color: String,
    pub border_color: String,
    pub use_img_label: bool,
    pub img_label_uri_hans: String,
    pub img_label_uri_hant: String,
    pub img_label_uri_hans_static: String,
    pub img_label_uri_hant_static: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vip {
    #[serde(rename = "type")]
    pub type_field: i64,
    pub status: i64,
    pub due_date: i64,
    pub vip_pay_type: i64,
    pub theme_type: i64,
    pub label: Label,
    pub avatar_subscript: i64,
    pub nickname_color: String,
    pub role: i64,
    pub avatar_subscript_url: String,
    pub tv_vip_status: i64,
    pub tv_vip_pay_type: i64,
    pub tv_due_date: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Label {
    pub path: String,
    pub text: String,
    pub label_theme: String,
    pub text_color: String,
    pub bg_style: i64,
    pub bg_color: String,
    pub border_color: String,
    pub use_img_label: bool,
    pub img_label_uri_hans: String,
    pub img_label_uri_hant: String,
    pub img_label_uri_hans_static: String,
    pub img_label_uri_hant_static: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wallet {
    pub mid: i64,
    pub bcoin_balance: i64,
    pub coupon_balance: i64,
    pub coupon_due_time: i64,
}
