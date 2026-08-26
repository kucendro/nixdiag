/**
  Upstream table for the mesh vhosts; annotations attach to nginx.

  #: unit epstein/nginx
  #: scope mesh
*/
{
  #: -> diddy/grafana grafana :3000 name=grafana@ts:443
  grafana = "diddy.ts.example:3000";
}
