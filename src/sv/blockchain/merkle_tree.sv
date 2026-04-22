//==============================================================================
// NEX2426 Merkle Tree - Hardware Implementation
// Binary Merkle Tree for blockchain integrity verification
//==============================================================================

`include "nex2426_pkg.svh"

module merkle_tree #(
    parameter TREE_DEPTH = 8,
    parameter LEAF_COUNT = 256,
    parameter HASH_SIZE = 512
)(
    input  logic                    clk,
    input  rst_n,
    input  logic                    start,
    input  logic [HASH_SIZE-1:0]    leaf_data,
    input  logic [7:0]              leaf_index,
    input  logic                    leaf_valid,
    output logic                    ready,
    output logic [HASH_SIZE-1:0]    root_hash,
    output logic                    root_valid,
    output logic                    verification_result,
    output logic                    verification_valid
);
    
    import nex2426_pkg::*;
    
    // Tree structure
    logic [HASH_SIZE-1:0] tree_nodes [2*LEAF_COUNT-1:0];
    logic [7:0] current_leaf;
    logic building_tree;
    logic tree_built;
    
    // Hash computation engine
    hash_core hash_engine (
        .clk(clk),
        .rst_n(rst_n),
        .start(leaf_valid),
        .mode(MODE_HASH),
        .key(256'h0), // Use zero key for Merkle tree
        .input_data(leaf_data),
        .cost(1),
        .done(),
        .hash_out(tree_nodes[leaf_index]),
        .status()
    );
    
    // Tree building state machine
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            current_leaf <= 0;
            building_tree <= 1'b0;
            tree_built <= 1'b0;
            ready <= 1'b1;
            root_valid <= 1'b0;
            verification_valid <= 1'b0;
        end else if (start && ready) begin
            ready <= 1'b0;
            building_tree <= 1'b1;
            current_leaf <= 0;
            tree_built <= 1'b0;
        end else if (building_tree && leaf_valid) begin
            current_leaf <= current_leaf + 1;
            
            if (current_leaf == LEAF_COUNT - 1) begin
                building_tree <= 1'b0;
                tree_built <= 1'b1;
            end
        end else if (tree_built) begin
            // Build internal nodes
            build_internal_nodes();
            tree_built <= 1'b0;
            ready <= 1'b1;
            root_valid <= 1'b1;
        end
    end
    
    // Build internal nodes of Merkle tree
    task build_internal_nodes;
        integer level, node_count, parent_idx, child1_idx, child2_idx;
        logic [HASH_SIZE-1:0] combined_hash;
    begin
        for (level = 0; level < TREE_DEPTH; level++) begin
            node_count = LEAF_COUNT >> (level + 1);
            for (parent_idx = 0; parent_idx < node_count; parent_idx++) begin
                child1_idx = (1 << level) * parent_idx;
                child2_idx = child1_idx + (1 << level);
                
                combined_hash = tree_nodes[child1_idx] ^ tree_nodes[child2_idx];
                
                // Hash the combined data
                hash_engine.start = 1'b1;
                hash_engine.input_data = combined_hash;
                // Wait for hash completion (simplified)
                #10;
                tree_nodes[parent_idx] = hash_engine.hash_out;
            end
        end
        root_hash = tree_nodes[0];
    end
    endtask
    
    // Verification logic
    logic verifying;
    logic [7:0] verify_path_idx;
    logic [HASH_SIZE-1:0] verify_hash;
    logic [HASH_SIZE-1:0] expected_root;
    
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            verifying <= 1'b0;
            verification_valid <= 1'b0;
            verification_result <= 1'b0;
        end else if (leaf_valid && root_valid && !verifying) begin
            verifying <= 1'b1;
            verify_path_idx = leaf_index;
            verify_hash = leaf_data;
            expected_root = root_hash;
        end else if (verifying) begin
            // Walk up the tree and verify
            logic [7:0] parent_idx;
            logic [HASH_SIZE-1:0] sibling_hash;
            logic [HASH_SIZE-1:0] combined_hash;
            
            parent_idx = verify_path_idx >> 1;
            
            if (verify_path_idx % 2 == 0) begin
                // Current node is left child
                sibling_hash = tree_nodes[parent_idx + 1];
                combined_hash = verify_hash ^ sibling_hash;
            end else begin
                // Current node is right child
                sibling_hash = tree_nodes[parent_idx - 1];
                combined_hash = sibling_hash ^ verify_hash;
            end
            
            // Hash combined data
            hash_engine.start = 1'b1;
            hash_engine.input_data = combined_hash;
            #10;
            verify_hash = hash_engine.hash_out;
            
            verify_path_idx = parent_idx;
            
            if (parent_idx == 0) begin
                verification_result = (verify_hash == expected_root);
                verification_valid <= 1'b1;
                verifying <= 1'b0;
            end
        end
    end
    
endmodule

//==============================================================================
// Blockchain Core Module
//==============================================================================

module blockchain_core #(
    parameter MAX_BLOCKS = 1024,
    parameter BLOCK_SIZE = 2048
)(
    input  logic                    clk,
    input  rst_n,
    input  logic                    add_block,
    input  logic [BLOCK_SIZE-1:0]    block_data,
    input  logic [511:0]            block_hash,
    input  logic [511:0]            prev_hash,
    output logic                    ready,
    output logic                    success,
    output logic [511:0]            chain_root,
    output logic                    chain_valid
);
    
    import nex2426_pkg::*;
    
    // Block storage
    logic [BLOCK_SIZE-1:0] block_storage [MAX_BLOCKS-1:0];
    logic [511:0] hash_storage [MAX_BLOCKS-1:0];
    logic [511:0] prev_hash_storage [MAX_BLOCKS-1:0];
    logic [9:0] block_count;
    logic adding_block;
    
    // Merkle tree for chain integrity
    logic [511:0] merkle_leaves [MAX_BLOCKS-1:0];
    
    merkle_tree #(.LEAF_COUNT(MAX_BLOCKS)) chain_merkle (
        .clk(clk),
        .rst_n(rst_n),
        .start(adding_block),
        .leaf_data(block_hash),
        .leaf_index(block_count),
        .leaf_valid(adding_block),
        .ready(),
        .root_hash(chain_root),
        .root_valid(chain_valid),
        .verification_result(),
        .verification_valid()
    );
    
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            block_count <= 0;
            adding_block <= 1'b0;
            ready <= 1'b1;
            success <= 1'b0;
        end else if (add_block && ready) begin
            ready <= 1'b0;
            adding_block <= 1'b1;
            success <= 1'b0;
            
            // Verify block linkage
            if (block_count == 0 || prev_hash_storage[block_count-1] == prev_hash) begin
                block_storage[block_count] <= block_data;
                hash_storage[block_count] <= block_hash;
                prev_hash_storage[block_count] <= prev_hash;
                merkle_leaves[block_count] <= block_hash;
                
                block_count <= block_count + 1;
                success <= 1'b1;
            end
            
            adding_block <= 1'b0;
            ready <= 1'b1;
        end
    end
    
endmodule
