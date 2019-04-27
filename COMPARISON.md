# Comparison to the original SeqBox implementation/design

The original SeqBox implementation and format do not support repairing of data, only sector level recoverability.

Blockyarchive supports both SeqBox and EC-SeqBox, while the original implementation only supports the SeqBox specification.

Blockyarchive is also more robust compared to the original SeqBox implementation, as it does not assume the SBX container to be well formed, and makes as few assumptions about the SBX container as possible.

blkar is overall based around [osbx](https://github.com/darrenldl/ocaml-SeqBox), but much more optimized.
