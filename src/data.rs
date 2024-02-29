#[derive(Clone, Debug, Default, PartialEq)]
pub struct PcbDataType {
    // Software version
    pub version: String,

    // sysop display name
    pub sysop: String,
    // sysop local password
    pub password: String,

    // true if use sysop real name instead of 'SYSOP'
    pub use_real_name: bool,

    /// node number of this node
    pub node_number: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UserRecord {
    pub name: String,
    pub city: String,
    pub password: String,

    pub bus_data_phone: String,
    pub home_voice_phone: String,

    pub last_date_on: u16,
    pub last_time_on: String,

    pub expert_mode: bool,

    /// Protocol (A->Z)
    pub protocol: char,

    // packedbyte     PackedFlags;        /* Bit packed flags */
    // datetype       DateLastDirRead;    /* Date for Last DIR Scan (most recent file) */
    pub security_level: i32,

    /// Number of times the caller has connected
    pub num_times_on: usize,

    /// Page length when display data on the screen
    pub page_len: i32,

    pub num_uploads: i32,
    pub num_downloads: i32,

    pub daily_downloaded_bytes: usize,

    pub user_comment: String,
    pub sysop_comment: String,

    /// Number of minutes online
    pub elapsed_time_on: i32,

    // unsigned short RegExpDate;         /* Julian date for Registration Expiration Date */
    // short          ExpSecurityLevel;   /* Expired Security Level */
    // unsigned short LastConference;     /* Number of the conference the caller was in */
    total_dl_bytes: usize,
    total_ul_bytes: usize,
    // bool           DeleteFlag;         /* 1=delete this record, 0=keep */
    // long           RecNum;             /* Record Number in USERS.INF file */
    // packedbyte2    Flags;
    // char           Reserved[8];        /* Bytes 390-397 from the USERS file */
    // unsigned long  MsgsRead;           /* Number of messages the user has read in PCB */
    // unsigned long  MsgsLeft;           /* Number of messages the user has left in PCB */
    alias_support: bool,
    alias: String,

    address_support: bool,
    // address: AddressType,

    // bool           PasswordSupport;
    // passwordtypez  PwrdHistory;
    // bool           VerifySupport;
    // char           Verify[26];
    // bool           StatsSupport;
    // callerstattype Stats;
    // bool           NotesSupport;
    // notestypez     Notes;
    // bool           AccountSupport;
    // accounttype    Account;
    // bool           QwkSupport;
    // qwkconfigtype  QwkConfig;
}
