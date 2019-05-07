macro_rules! block_pred_same_ver_uid {
    (
        $block:expr
    ) => {{
        use sbx_block::Block;

        let version = $block.get_version();
        let uid = $block.get_uid();
        move |block: &Block| -> bool { block.get_version() == version && block.get_uid() == uid }
    }};
}
