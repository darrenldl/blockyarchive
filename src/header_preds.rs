macro_rules! header_pred_same_ver_uid {
    (
        $block:expr
    ) => {{
        use sbx_block::Header;

        let version = $block.get_version();
        let uid = $block.get_uid();
        move |header: &Header| -> bool { header.version == version && header.uid == uid }
    }};
}
