//==============================================================================
// NEX2426 Blockchain Interface
// Standard interface for blockchain operations
//==============================================================================

`ifndef BLOCKCHAIN_IF_SVH
`define BLOCKCHAIN_IF_SVH

`include "nex2426_pkg.svh"

interface blockchain_if;
    import nex2426_pkg::*;
    
    logic                    clk;
    logic                    rst_n;
    logic                    add_block;
    logic [2047:0]           block_data;
    logic [511:0]            block_hash;
    logic [511:0]            prev_hash;
    logic                    ready;
    logic                    success;
    logic [511:0]            chain_root;
    logic                    chain_valid;
    
    modport master (
        output add_block,
        output block_data,
        output block_hash,
        output prev_hash,
        input  ready,
        input  success,
        input  chain_root,
        input  chain_valid
    );
    
    modport slave (
        input  add_block,
        input  block_data,
        input  block_hash,
        input  prev_hash,
        output ready,
        output success,
        output chain_root,
        output chain_valid
    );
    
endinterface

`endif // BLOCKCHAIN_IF_SVH
