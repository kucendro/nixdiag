{
  stateVersions = [
    "25.05"
    "26.05"
  ];
  module.services.nixdiag.serve = {
    enable = true;
    virtualHost = "docs.test";
  };
}
