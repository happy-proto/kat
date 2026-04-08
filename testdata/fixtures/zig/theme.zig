const std = @import("std");

pub fn main() void {
    const theme = "Dracula";
    std.debug.print("{s}\n", .{theme});
}
