library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity theme_preview is
  port (
    clk     : in std_logic;
    enabled : in std_logic;
    accent  : out std_logic_vector(7 downto 0)
  );
end entity theme_preview;

architecture rtl of theme_preview is
  signal counter : unsigned(7 downto 0) := x"00";
begin
  process (clk)
  begin
    if rising_edge(clk) then
      if enabled = '1' then
        counter <= counter + 1;
      end if;
    end if;
  end process;

  accent <= std_logic_vector(counter);
end architecture rtl;
