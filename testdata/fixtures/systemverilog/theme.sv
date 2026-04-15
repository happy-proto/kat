module theme_preview (
  input logic clk,
  input logic rst_n,
  output logic active
);
  always_ff @(posedge clk or negedge rst_n) begin
    if (!rst_n) begin
      active <= 1'b0;
    end else begin
      active <= 1'b1;
    end
  end
endmodule
