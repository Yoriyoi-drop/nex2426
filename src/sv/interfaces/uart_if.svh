//==============================================================================
// NEX2426 UART Interface
// Standard UART interface for debugging and communication
//==============================================================================

`ifndef UART_IF_SVH
`define UART_IF_SVH

interface uart_if;
    logic                    clk;
    logic                    rst_n;
    logic                    tx;
    logic                    rx;
    logic                    tx_valid;
    logic                    tx_ready;
    logic [7:0]             tx_data;
    logic                    rx_valid;
    logic                    rx_ready;
    logic [7:0]             rx_data;
    
    modport master (
        output tx_valid,
        input  tx_ready,
        output tx_data,
        input  rx_valid,
        output rx_ready,
        input  rx_data
    );
    
    modport slave (
        input  tx_valid,
        output tx_ready,
        input  tx_data,
        output rx_valid,
        input  rx_ready,
        output rx_data
    );
    
endinterface

`endif // UART_IF_SVH
