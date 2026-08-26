# Fixture "flake". Never evaluated as a real flake — the nixdiag checks build
# its hosts directly with nixosSystem, and nixdiag's module renderer parses
# this file textually for targetModule entries.
{
  hosts = {
    diddy = {
      targetModule = ./hosts/diddy;
    };
    epstein = {
      targetModule = ./hosts/epstein;
    };
  };
}
