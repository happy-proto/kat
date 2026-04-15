package theme_pkg;
  typedef enum logic [1:0] {
    THEME_DARK,
    THEME_LIGHT
  } theme_kind_t;
endpackage

module theme_preview #(parameter int WIDTH = 8) (
  input logic clk,
  input logic rst_n,
  output logic [WIDTH-1:0] accent
);
  import theme_pkg::*;

  always_ff @(posedge clk or negedge rst_n) begin
    if (!rst_n) begin
      accent <= '0;
    end else begin
      accent <= WIDTH'(8'hbd);
    end
  end
endmodule
