import { UmiPlugin } from '@metaplex-foundation/umi';
import { createMerkleTreeStorageProgram } from './generated';

export const merkleTreeStorage = (): UmiPlugin => ({
  install(umi) {
    umi.programs.add(createMerkleTreeStorageProgram(), false);
  },
});
