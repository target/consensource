'use strict'

const m = require('mithril')
const AuthService = require('App/services/auth')
const standardsService = require('App/services/standards')
const agentService = require('App/services/agent')
var DatePicker = require('mithril-datepicker')


var StandardPayloadData = {
    id: "",
    name: "",
    version: "",
    link: "",
    description: "",
    approvalDate: new Date().getTime() / 1000,

    submitting: false,
    errorMsg: null,

    setID: (id) => {
        StandardPayloadData.id = id
    },

    setName: (name) => {
        StandardPayloadData.name = name
    },

    setVersion: (version) => {
        StandardPayloadData.version = version
    },

    setLink: (link) => {
        StandardPayloadData.link = link
    },

    setDescription: (description) => {
        StandardPayloadData.description = description
    },

    setApprovalDate: (approvalDate) => {
        StandardPayloadData.approvalDate = approvalDate
    },

    clear: () => {
      StandardPayloadData.id = ''
      StandardPayloadData.name = ''
      StandardPayloadData.version = ''
      StandardPayloadData.link = ''
      StandardPayloadData.description = ''
      StandardPayloadData.approvalDate = new Date().getTime() / 1000
      StandardPayloadData.submitting = false
      StandardPayloadData.errorMsg = null
    },

    submitCreateStandard: (organizationId) => {
        StandardPayloadData.submitting = true
        return AuthService.getSigner()
            .then((signer) =>
              standardsService.createStandard(
                  StandardPayloadData,
                  organizationId,
                  signer)
            )
            .then(() => {
                StandardPayloadData.clear()
                m.route.set('/standardsList')
            })
            .catch((errorMsg) => {
                StandardPayloadData.errorMsg = errorMsg;
                StandardPayloadData.submitting = false
                m.redraw()
            })
    },
    submitUpdateStandard: (organizationId) => {
        StandardPayloadData.submitting = true
        return AuthService.getSigner()
            .then((signer) =>
              standardsService.updateStandard(
                  StandardPayloadData,
                  organizationId,
                  signer)
            )
            .then(() => {
                StandardPayloadData.clear()
                m.route.set('/standardsList')
            })
            .catch((errorMsg) => {
                StandardPayloadData.errorMsg = errorMsg;
                StandardPayloadData.submitting = false
                m.redraw()
            })
    }
}

var StandardCreate = {
    _viewName: 'StandardCreate',
    oninit: (vnode) => {

      StandardPayloadData.clear()
      vnode.state.loading = true
      vnode.state.agent = null

      return AuthService.getUserData()
        .then((user) => Promise.all([
          agentService.fetchAgent(user.public_key),
        ])
        .then(([agent]) => {
           vnode.state.loading = false
           vnode.state.agent = agent.data
           m.redraw()
        })
        .catch((e) => {
          console.log(e)
          vnode.state.loading = false
        }))
    },
    view: (vnode) =>  {
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
          m('h5', 'Standard Information'),
          m('div.form', [
            StandardPayloadData.errorMsg ? m('p.text-danger', StandardPayloadData.errorMsg) : null,
            m('div.form-group', [
                m('label[for=name]', 'Standard Name'),
                m("input.form-control[type=text]", { oninput: m.withAttr("value", StandardPayloadData.setName), value: StandardPayloadData.name }),

                m('label[for=version]', 'Version'),
                m("input.form-control[type=text]", { oninput: m.withAttr("value", StandardPayloadData.setVersion), value: StandardPayloadData.version }),

                m('label[for=description]', 'Description'),
                m("input.form-control[type=text]", { oninput: m.withAttr("value", StandardPayloadData.setDescription), value: StandardPayloadData.description }),

                m('label[for=link]', 'Link'),
                m("input.form-control[type=text]", { oninput: m.withAttr("value", StandardPayloadData.setLink), value: StandardPayloadData.link }),

                m('label[for=approvalDate]', 'Approval Date'),
                m('br'),
                m(DatePicker, {onchange: (chosenDate) => {
                    StandardPayloadData.setApprovalDate(chosenDate.getTime() / 1000)
                }})
              ]),
            m("button.btn.btn-primary", {
                onclick: () => {
                  StandardPayloadData.submitCreateStandard(vnode.state.agent.organization.id)
                },
                disabled: StandardPayloadData.submitting,
            }, "Create Standard"),
          ]),
        ]
      } else {
        return [m('p', "Unable to load details")]
        }
      }
  }

var StandardUpdate = {
      _viewName: 'StandardUpdate',
      oninit: (vnode) => {
        StandardPayloadData.clear()
        vnode.state.loading = true
        vnode.state.agent = null
        return AuthService.getUserData()
          .then((user) => Promise.all([
            agentService.fetchAgent(user.public_key)])
          .then(([agent]) => {
             vnode.state.agent = agent.data
             standardsService.fetchStandard(agent.data.organization.id, m.route.param("standard_id"))
             .then((standard) => {
                vnode.state.loading = false
                vnode.state.standard = standard
                m.redraw()
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
      view: (vnode) =>  {
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
            m('h5', 'Standard Information'),
            m('div.form', [
              StandardPayloadData.errorMsg ? m('p.text-danger', StandardPayloadData.errorMsg) : null,
              m('div.form-group', [
                  m('label[for=name]', 'Standard Name'),
                  m("input.form-control-plaintext[type=text][readonly=true]", { value: vnode.state.standard.name }),

                  m('label[for=version]', 'Version'),
                  m("input.form-control[type=text]", { oninput: m.withAttr("value", StandardPayloadData.setVersion), value: StandardPayloadData.version }),

                  m('label[for=description]', 'Description'),
                  m("input.form-control[type=text]", { oninput: m.withAttr("value", StandardPayloadData.setDescription), value: StandardPayloadData.description }),

                  m('label[for=link]', 'Link'),
                  m("input.form-control[type=text]", { oninput: m.withAttr("value", StandardPayloadData.setLink), value: StandardPayloadData.link }),

                  m('label[for=approvalDate]', 'Approval Date'),
                  m('br'),
                  m(DatePicker, {onchange: (chosenDate) => {
                      StandardPayloadData.setApprovalDate(chosenDate.getTime() / 1000)
                  }})
                ]),
              m("button.btn.btn-primary", {
                  onclick: () => {
                    StandardPayloadData.setID(m.route.param("standard_id"))
                    StandardPayloadData.submitUpdateStandard(vnode.state.agent.organization.id)
                  },
                  disabled: StandardPayloadData.submitting,
              }, "Create New Version"),
            ]),
          ]
        } else {
          return [m('p', "Unable to load details")]
          }
        }
    }

var StandardList =  {
    _viewName: 'StandardList',
    view: (vnode) => [
        m('table.table', [
            m('thead.thead-dark', m('tr', [
                m('th[scope=col]', "Standard Name"),
                m('th[scope=col]', "Latest Version"),
                m('th[scope=col]', "Description"),
                m('th[scope=col]', "Link"),
                m('th[scope=col]', "Approval Date"),
                m('th[scope=col]', ""),

            ])),
            m('tbody',
                _renderRows(
                    vnode.state.standards,
                    ((standard) => m('tr', [
                        m('td', standard.name),
                        m('td', standard.versions[standard.versions.length - 1].version),
                        m('td', standard.versions[standard.versions.length - 1].description),
                        m('td', standard.versions[standard.versions.length - 1].external_link),
                        m('td', _renderTimestamp(standard.versions[standard.versions.length - 1].approval_date)),
                        m('td', m(`button.btn.btn-success.btn-sm[href=/standardsUpdate?standard_id=${standard.standard_id}]`,
                          {
                          oncreate: m.route.link,
                          }, "Create New Version"))
                    ])),
                    m('tr', vnode.state.noRecordsElement)))
        ])
    ],

    oninit: (vnode) => {
        vnode.state.standards = []
        vnode.state.loading = true
        vnode.state.noRecordsElement = m('td.text-center[colspan=3]', 'No Standards Found')
    },

    oncreate: (vnode) => {
        AuthService.getUserData()
          .then((user) => Promise.all([
          agentService.fetchAgent(user.public_key)])
        .then(([agent]) => {
          standardsService.loadStandards(agent.data.organization.id)
              .then((standards) => {
                standards.data.sort((a, b) => a.name > b.name)
                vnode.state.standards = standards.data
                vnode.state.loading = false
              })
              .catch(() => {
                  vnode.state.noRecordsElement =
                      m('td.text-center.text-danger[colspan=3]', 'Failed to fetch Standards')
              })
        })
        .catch((e) => {
          console.log(e)
        }))

    },
}

const _renderTimestamp = (timestamp) => {
  if (timestamp) {
    let d = new Date(timestamp * 1000)
    return `${d.toLocaleDateString()}`
  } else {
    return 'Unknown'
  }
}

const _renderRows = (items, renderer, emptyElement) => {
    if (items.length > 0) {
        return items.map(renderer)
    } else {
        return emptyElement
    }
}

module.exports = {
  StandardPayloadData,
  StandardCreate,
  StandardUpdate,
  StandardList
}
