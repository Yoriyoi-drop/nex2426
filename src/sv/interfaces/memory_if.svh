//==============================================================================
// NEX2426 Memory Interface
// Standard interface for memory operations
//==============================================================================

`ifndef MEMORY_IF_SVH
`define MEMORY_IF_SVH

`include "nex2426_pkg.svh"

interface memory_if;
    import nex2426_pkg::*;
    
    logic                    clk;
    logic                    rst_n;
    logic                    enable;
    logic                    write_enable;
    logic [11:0]             address;
    logic [63:0]             write_data;
    logic [63:0]             mask;
    logic [63:0]             read_data;
    logic                    valid;
    logic                    ready;
    
    modport master (
        output enable,
        output write_enable,
        output address,
        output write_data,
        output mask,
        input  read_data,
        input  valid,
        input  ready
    );
    
    modport slave (
        input  enable,
        input  write_enable,
        input  address,
        input  write_data,
        input  mask,
        output read_data,
        output valid,
        output ready
    );
    
endinterface

`endif // MEMORY_IF_SVH
