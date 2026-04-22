//==============================================================================
// NEX2426 Crypto Interface
// Standard interface for cryptographic operations
//==============================================================================

`ifndef CRYPTO_IF_SVH
`define CRYPTO_IF_SVH

interface crypto_if;
    import nex2426_pkg::*;
    
    logic                    clk;
    logic                    rst_n;
    logic                    start;
    operation_mode_t         mode;
    logic [255:0]            key;
    logic [511:0]            input_data;
    logic [31:0]             cost;
    logic                    done;
    logic [511:0]            output_data;
    status_code_t           status;
    
    modport master (
        output start,
        output mode,
        output key,
        output input_data,
        output cost,
        input  done,
        input  output_data,
        input  status
    );
    
    modport slave (
        input  start,
        input  mode,
        input  key,
        input  input_data,
        input  cost,
        output done,
        output output_data,
        output status
    );
    
endinterface

`endif // CRYPTO_IF_SVH
