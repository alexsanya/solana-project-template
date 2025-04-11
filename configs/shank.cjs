const path = require("path");
const { generateIdl } = require("@metaplex-foundation/shank-js");

const idlDir = path.join(__dirname, "..", "idls");
const binaryInstallDir = path.join(__dirname, "..", ".crates");
const programDir = path.join(__dirname, "..", "programs");

generateIdl({
  generator: "shank",
  programName: "merkle_tree_storage_program",
  programId: "TREEZwpvqQN6HVAAPjqhJAr8BuoGhXSx34jm9YV5DPB",
  idlDir,
  idlName: "merkle_tree_storage",
  binaryInstallDir,
  programDir: path.join(programDir, "merkle-tree-storage"),
});
