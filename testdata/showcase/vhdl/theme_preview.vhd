library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity preview_pipeline is
  generic (
    ThemeWidth : natural := 16
  );
  port (
    clk        : in std_logic;
    reset_n    : in std_logic;
    theme_name : in std_logic_vector(ThemeWidth - 1 downto 0);
    active     : out std_logic
  );
end entity preview_pipeline;

architecture rtl of preview_pipeline is
  type state_t is (idle, warmup, ready);
  signal state   : state_t := idle;
  signal counter : unsigned(3 downto 0) := (others => '0');
begin
  process (clk, reset_n)
  begin
    if reset_n = '0' then
      state <= idle;
      counter <= (others => '0');
    elsif rising_edge(clk) then
      case state is
        when idle =>
          state <= warmup;
        when warmup =>
          counter <= counter + 1;
          if counter = 3 then
            state <= ready;
          end if;
        when ready =>
          null;
      end case;
    end if;
  end process;

  active <= '1' when state = ready else '0';
end architecture rtl;
