'use strict'

const m = require('mithril')
const AuthService = require('App/services/auth')
const organizationService = require('App/services/organization')
const { Organization: OrganizationProto } = require('App/protobuf')
const isoLangCodes = require('App/views/common/ISO-639-1-language.json')

/**
 * Model/Controller for Organization Create Form
 */
var Organization = {
    name: "",
    type: 0,
    contact: OrganizationProto.Contact.create(),

    submitting: false,
    errorMsg: null,

    setName: (name) => {
        Organization.name = name
    },

    setType: (type) => {
        Organization.type = type
    },

    setContactName: (name) => {
        Organization.contact.name = name
    },

    setContactPhoneNumber: (phoneNumber) => {
        Organization.contact.phoneNumber = phoneNumber
    },
    setContactLanguageCode: (languageCode) => {
        Organization.contact.languageCode = languageCode
    },


    clear: () => {
        Organization.id = ''
        Organization.name = ''
        Organization.type = 0
        Organization.contact = OrganizationProto.Contact.create()
        Organization.submitting = false
        Organization.errorMsg = null
    },

    submit: () => {
        Organization.submitting = true
        return AuthService.getSigner()
            .then((signer) =>
                organizationService.createOrganization(
                    Organization.name,
                    Organization.type,
                    Organization.contact,
                    signer))
            .then(() => {
                Organization.clear()
                m.route.set('/profile')
                m.redraw()
            })
            .catch((errorMsg) => {
                Organization.errorMsg = errorMsg;
                Organization.submitting = false
            })
    }
}


var OrganizationCreate = {
    _viewName: 'OrganizationCreate',
    view: (vnode) => [
        m('div.form', [
            vnode.state.organization.errorMsg ? m('p.text-danger', vnode.state.organization.errorMsg) : null,
            m('div.form-group', [
                m('label[for=organizationName]', 'Name'),
                m("input.form-control[type=text]", { oninput: m.withAttr("value", vnode.state.organization.setName), value: vnode.state.organization.name }),
                m('h5', 'Contact Information'),
                m('label[for=contactName]', 'Contact Name'),
                m("input.form-control[type=text]", { oninput: m.withAttr("value", vnode.state.organization.setContactName), value: vnode.state.organization.contact.name }),
                m('label[for=contactPhoneNumber]', 'Phone Number'),
                m("input.form-control[type=text]", { oninput: m.withAttr("value", vnode.state.organization.setContactPhoneNumber), value: vnode.state.organization.contact.phoneNumber }),
                m('label[for=contactLanguageCode]', 'Language Code'),
                m("select.form-control", {
                  oninput: m.withAttr("value", vnode.state.organization.setContactLanguageCode),
                  value: vnode.state.organization.contact.languageCode
                }, isoLangCodes.map(({code, name}) => m('option', {value: code, text: name}))),

            ]),
            m("button.btn.btn-primary", {
                onclick: vnode.state.organization.submit,
                disabled: vnode.state.organization.submitting,
            }, "Create"),
        ]),
    ]
}

const _renderRows = (items, renderer, emptyElement) => {
    if (items.length > 0) {
        return items.map(renderer)
    } else {
        return emptyElement
    }
}

var OrganizationList = {
    _viewName: 'OrganizationList',
    view: (vnode) => [
        m('table.table', [
            m('thead.thead-dark', m('tr', [
                m('th[scope=col]', "Organization Id"),
                m('th[scope=col]', "Organization Name"),
                m('th[scope=col]', "Organization Type"),
            ])),
            m('tbody',
                _renderRows(
                    vnode.state.organizations,
                    ((organization) => m('tr', [
                        m('td', organization.id),
                        m('td', organization.name),
                        m('td', organization.organization_type)
                    ])),
                    m('tr', vnode.state.noRecordsElement)))
        ])
    ],

    oninit: (vnode) => {
        vnode.state.organizations = []
        vnode.state.loading = true
        vnode.state.noRecordsElement = m('td.text-center[colspan=3]', 'No Organizations Found')
    },

    oncreate: (vnode) => {
        organizationService.loadOrganizations()
            .then((organizations) => {
              organizations.data.sort((a, b) => a.name > b.name)
              vnode.state.organizations = organizations.data
              vnode.state.loading = false
            })
            .catch(() => {
                vnode.state.noRecordsElement =
                    m('td.text-center.text-danger[colspan=3]', 'Failed to fetch Organizations')
            })
    },
}

module.exports = {
    Organization,
    OrganizationCreate,
    OrganizationList,
}
