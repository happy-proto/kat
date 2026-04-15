module theme_preview(
  input wire clk,
  output reg active
);
  always @(posedge clk) begin
    active <= 1'b1;
  end
endmodule
