# Fixture "flake". Never evaluated as a real flake — the nixdiag checks build
# its hosts directly with nixosSystem, and nixdiag's module renderer parses
# this file textually for targetModule entries.
{
  hosts = {
    luna = {
      targetModule = ./hosts/luna;
    };
    sol = {
      targetModule = ./hosts/sol;
    };
  };
}
