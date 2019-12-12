'use strict'

const m = require('mithril')
const AuthService = require('App/services/auth')
const accreditCertifyingBodyService = require('App/services/accreditation')
const agentService = require('App/services/agent')
const organizationService = require('App/services/organization')
const standardsService = require('App/services/standards')
var DatePicker = require('mithril-datepicker')

const Standards = {
    list: [],
    get: (vnode) => {
        if (vnode.state.standards !== null) {
            if (Standards.list.length !== vnode.state.standards.length) {
                Standards.list = []
                for (let standard of vnode.state.standards) {
                    Standards.list.push(m('option', {
                        value: standard.standard_id,
                        text: standard.name,
                    }))
                }
            }
        }
    }
}

var AccreditCertifyingBodyData = {
    certifyingBodyId: "",
    standardId: "",
    validTo: 0,
    validFrom: new Date().getTime() / 1000,
    submitting: false,
    errorMsg: null,

    setCertifyingBodyId: (certifyingBodyId) => {
        AccreditCertifyingBodyData.certifyingBodyId = certifyingBodyId
    },

    setStandardId: (standardId) => {
        AccreditCertifyingBodyData.standardId = standardId
    },

    setValidTo: (timestamp) => {
        AccreditCertifyingBodyData.validTo = timestamp
    },

    setValidFrom: (timestamp) => {
        AccreditCertifyingBodyData.validFrom = timestamp
    },

    clear: () => {
        AccreditCertifyingBodyData.certifyingBodyId = ''
        AccreditCertifyingBodyData.standardId = ''
        AccreditCertifyingBodyData.validTo = 0
        AccreditCertifyingBodyData.validFrom = new Date().getTime() / 1000
        AccreditCertifyingBodyData.submitting = false
        AccreditCertifyingBodyData.errorMsg = null
    },

    submit: (certifyingBodyId, standardsBodyId) => {
        AccreditCertifyingBodyData.submitting = true
        return AuthService.getSigner()
            .then((signer) =>
                accreditCertifyingBodyService.accreditCertifyingBody(
                    AccreditCertifyingBodyData,
                    standardsBodyId,
                    certifyingBodyId,
                    signer)
                )
                .then(() => {
                    AccreditCertifyingBodyData.clear()
                    m.route.set('/certifyingBodyList')
                })
                .catch((errorMsg) => {
                    AccreditCertifyingBodyData.errorMsg = errorMsg;
                    AccreditCertifyingBodyData.submitting = false
                    m.redraw()
                })
    }
}

var AccreditCertifyingBody = {
    _viewName: 'AccreditCertifyingBody',
    oninit: (vnode) => {

        AccreditCertifyingBodyData.clear()
        vnode.state.loading = true
        vnode.state.agent = null
        vnode.state.organization = null
        vnode.state.standards = null


        return AuthService.getUserData()
            .then((user) => Promise.all([
                agentService.fetchAgent(user.public_key),
                organizationService.fetchOrganization(m.route.param("organization_id")),
            ])
            .then(([agent, organization]) => {
                vnode.state.loading = false
                vnode.state.organization = organization.data
                vnode.state.agent = agent.data
                standardsService.loadStandards(agent.data.organization.id)
                .then((standards) => {
                    standards.data.sort((a, b) => a.name > b.name)
                    vnode.state.standards = standards.data
                    m.redraw()
                })
                .catch(() => {
                    vnode.state.noRecordsElement =
                        m('td.text-center.text-danger[colspan=3]', 'Failed to fetch Standards')
                    vnode.state.loading = false
                })
                .catch((e) => {
                    console.log(e)
                    vnode.state.loading = false
                })
            })
            .catch((e) => {
                console.log(e)
                vnode.state.loading = false
            }))
        },

        view: (vnode) => {
            if (vnode.state.loading) {
                return m('p', 'Loading...')
            } else if (vnode.state.agent) {
                DatePicker.localize({
                    weekStart: 1,
                    prevNextTitles: ['1M', '1A', '10A'],
                    formatOptions: {
                      day: 'numeric',
                      month: 'numeric',
                      year: 'numeric'
                    }
                })
            return [
                m('h6', 'Accreditation Information'),
                m('div.form', [
                    AccreditCertifyingBodyData.errorMsg ? m('p.text-danger', AccreditCertifyingBodyData.errorMsg) : null,
                    m('div.form-group', [
                        m('div.form-group.row', [
                        m('label[for=standardId]', 'Standard ID'),
                        m("select.form-control.mr-2", {
                            oninit: Standards.get(vnode),
                            oninput: m.withAttr("value", AccreditCertifyingBodyData.setStandardId),
                            value: AccreditCertifyingBodyData.standardId
                        }, Standards.list),
                    ]),
                    m('div.form-group.row', [
                        m('div.col', [
                        m('label', 'Valid from'),
                        m(DatePicker, {
                            onchange: (chosenDate) => {
                                AccreditCertifyingBodyData.setValidFrom(chosenDate.getTime() / 1000)
                                }
                            }),
                        ]),
                    ]),
                    m('div.form-group.row', [
                        m('div.col', [
                        m('label', 'Valid to'),
                        m(DatePicker, {
                            onchange: (chosenDate) => {
                                AccreditCertifyingBodyData.setValidTo(chosenDate.getTime() / 1000)
                                }
                            }),
                        ]),
                    ]),
                    m('div.form-group', [
                        m('div.form-group.row', [
                        m('label[for=certifyingBodyName]', 'Certifying Body Name'),
                        m("input.form-control-plaintext[type=text][readonly=true]",  {value: vnode.state.organization.name }),
                        ]),
                    ]),
                ]),

                m("button.btn.btn-primary", {
                    onclick: () => {
                        AccreditCertifyingBodyData.setCertifyingBodyId(vnode.state.organization.id)
                        AccreditCertifyingBodyData.submit(vnode.state.organization.id, vnode.state.agent.organization.id)
                    },
                    disabled: AccreditCertifyingBodyData.submitting,
                }, "Accredit Certifying Body"),
            ]),
        ]
    } else {
        return [m('p', "Unable to load details")]
    }
}
}

module.exports = {
    AccreditCertifyingBodyData,
    AccreditCertifyingBody
}
