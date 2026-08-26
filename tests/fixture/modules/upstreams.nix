/**
  Upstream table for the mesh vhosts; annotations attach to nginx.

  #: unit sol/nginx
  #: scope mesh
*/
{
  #: -> luna/grafana grafana :3000 name=grafana@ts:443
  grafana = "luna.ts.example:3000";
}
