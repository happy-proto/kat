`define THEME_WIDTH 8

module theme_preview(
  input wire clk,
  input wire rst_n,
  output reg [`THEME_WIDTH-1:0] accent
);
  always @(posedge clk or negedge rst_n) begin
    if (!rst_n) begin
      accent <= 8'h00;
    end else begin
      accent <= 8'hbd;
    end
  end
endmodule
