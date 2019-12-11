const crypto = require("crypto");

const FAMILY_NAME = "consensource";
const FAMILY_VERSION = "0.1";
const AGENT = "00";
const CERTIFICATE = "01";
const ORGANIZATION = "02";
const STANDARD = "03";
const CERTIFICATE_REQUEST = "04";

const PREFIX_SIZE = 6;
const RESERVED_SPACE = "00";

function hash(object, num) {
  let sha = crypto.createHash("sha256");
  return sha
    .update(object)
    .digest("hex")
    .substring(0, num);
}

const FAMILY_NAMESPACE = hash(FAMILY_NAME, PREFIX_SIZE);
const AGENT_ADDRESS_PREFIX = FAMILY_NAMESPACE + RESERVED_SPACE + AGENT;
const ORGANIZATION_ADDRESS_PREFIX =
  FAMILY_NAMESPACE + RESERVED_SPACE + ORGANIZATION;

module.exports = {
  getFamilyNamespacePrefix() {
    return FAMILY_NAMESPACE;
  },
  makeAgentAddress(agentPublicKey) {
    return AGENT_ADDRESS_PREFIX + hash(agentPublicKey, 60);
  },
  makeOrganizationAddress(organizationID) {
    return (
      FAMILY_NAMESPACE +
      RESERVED_SPACE +
      ORGANIZATION +
      hash(organizationID, 60)
    );
  },
  makeCertificateAddress(certificateID) {
    return (
      FAMILY_NAMESPACE + RESERVED_SPACE + CERTIFICATE + hash(certificateID, 60)
    );
  },
  makeCertificateRequestAddress(certificateRequestId) {
    return (
      FAMILY_NAMESPACE +
      RESERVED_SPACE +
      CERTIFICATE_REQUEST +
      hash(certificateRequestId, 60)
    );
  },
  makeStandardAddress(standardId) {
    return FAMILY_NAMESPACE + RESERVED_SPACE + STANDARD + hash(standardId, 60);
  },

  familyName: FAMILY_NAME,
  familyVersion: FAMILY_VERSION,

  agentAddressPrefix: AGENT_ADDRESS_PREFIX,
  organizationAddressPrefix: ORGANIZATION_ADDRESS_PREFIX
};
