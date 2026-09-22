d2:
if (d2.override.__functionArgs or { }) ? withImageSupport then
  d2.override { withImageSupport = false; }
else
  d2
