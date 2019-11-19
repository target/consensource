'use strict'

const m = require('mithril')
const retailerSearchService = require('App/services/retailer_search')


const _searchForm = (vnode) => m('.form-row', [
  m('div.col-10',
    m(`input.form-control.searchBar[type=text][name="searchFactories"][placeholder="Search by Factory Name, Supplier ID, Certification Type or Location"]`, {
      oninput: m.withAttr('value', (v) => {
        vnode.state.value = v
      }),
      value: vnode.state.value
    })),
  m('div.col-2',
    m('button.btn.btn-success#searchFactory-btn',
      { onclick: () => _doSearch(vnode) },
      m('img.search-icon[src=/assets/images/search-icon.svg]'))
  )
])


const _renderRows = (items, renderer, emptyElement) => {
  if (items && items.length > 0) {
    return items.map(renderer)
  } else {
    return emptyElement
  }
}

const _renderLocation = (address) => [
  address.city, ", ",
  address.state_province,
  !address.state_province ? '' : ", ",
  address.country
]

function toggleFactoryDetails(vnode, index) {
  let toggle = document.querySelectorAll(`.toggle-factory-${index}`)
  Object.values(toggle).map((t) => {
    t.classList.toggle("show");
    t.classList.toggle("hide");
    return ""
  })

  let details_div = document.querySelector(`#factory-details-${index}`)
  details_div.classList.toggle("view");
  details_div.classList.toggle("hide");

  let row = document.querySelector(`#factory-${index}`)
  row.classList.toggle("selected");
}

const FactoryTable = {
  _viewName: 'FactoryTable',
  view: (vnode) => [
    m('table.table.table-bordered.factory-table', [
      m('thead.thead-dark', m('tr', [
        m('th[scope=col]', "Factory Name"),
        m('th[scope=col]', "Supplier ID"),
        m('th[scope=col]', "Certification Type(s)"),
        m('th[scope=col]', "Location"),
        m('th[scope=col]', "Details"),
      ])),
      m('tbody',
        _renderRows(
          vnode.attrs.factories,
          ((factory, index) => [m(`tr.select-row.factory-info#factory-${index}`, [
            m('td.pl-5', factory.name),
            m('td.pl-5', factory.target_id ? factory.target_id : "Not Current Supplier"),
            m('td.pl-5', _renderCertificationTypes(factory.certificates)),
            m('td.pl-5', _renderLocation(factory.address)),
            m(`td.pl-5.toggle-factory-details`, { onclick: (vnode) => toggleFactoryDetails(vnode, index) }, [
              m(`span.view-toggle-text.toggle-factory-${index}.show`, "View ",
                m('img.arrow-down[src=/assets/images/chevron-black.svg]')),
              m(`span.toggle-factory-${index}.hide`, "Hide ",
                m('img[src=/assets/images/chevron-black.svg]'))
            ]),
          ]),
          m(`tr.factory-details.hide#factory-details-${index}`, _renderFactoryDetails(factory, index))]),
          m('tr', m('td[colspan=5]', 'No factories found for the specified details.')))

      )
    ])
  ]
}
const _renderCertificationTypes = (certificates) => {
  let certificateTypes = certificates.map(cert => cert.standard_name)

  if (certificateTypes.length === 0) {
    return "No Certificates Found"
  }

  let unique_types = new Set(certificateTypes)
  return [...unique_types].map((certType, index) => `${certType}${index === unique_types.size - 1 ? '' : ', '}`)
}

const _renderFactoryDetails = (factory) => [
  m('td.factory-details.factory-profile-link',
    m(`a[href=/certifications/factoryProfile?factory_id=${factory.id}].factory-profile-link`,
      { oncreate: m.route.link }, "See this factory's profile "),
    m('img[src=/assets/images/arrow-go.svg]')
  ),
  m('td.factory-details', ""),
  m('td.factory-details', m('p.factory-details-subtitle', 'Current Certifications'),
    _renderCertificates(factory.certificates)),
  m('td.factory-details', m('p.factory-details-subtitle', 'Contact Information'),
    _renderContactInfo(factory.address, factory.contacts[0])),
  m('td.factory-details', "")]

const _renderContactInfo = (address, contact) => m('span.factory-contact-info', [
  address.street_line_1, m('br'),
  address.street_line_2,
  !address.street_line_2 ? '' : m('br'),
  address.city, ', ',
  address.state_province,
  !address.state_province ? '' : ", ",
  address.postal_code,
  !address.postal_code ? '' : m('br'),
  address.country, m('br'),
  contact.phone_number, m('br'), m('br'),
  contact.name
])

const _renderCertificates = (certData) => {
  if (certData.length === 0) {
    return "No Certificates Found"
  }
  return m('ul.list-unstyled', [certData.map(cert => m('li', cert.id))])
}

const SearchResults = {
  _viewName: 'SearchResults',

  oninit: (vnode) => {
    vnode.attrs.factories = vnode.attrs.factories || []
    vnode.state.selectedFactory = null
  },
  onupdate: (vnode) => {
    vnode.state.selectedFactory = null
  },
  view: (vnode) => [
    m('.row.factory-results.mb-3', m(FactoryTable, {
      factories: vnode.attrs.factories
    }))
  ]
}

const _match = (s, partial) => {
  if (s) {
    return s.toLowerCase().startsWith(partial.toLowerCase())
  } else {
    return false
  }
}

const _doSearch = (vnode) => {
  let searchInput = vnode.state.value
  let ss = vnode.state.searchSpace
  let results = []
  results = results.concat(ss.filter((factory) => _match(factory.name, searchInput)))
  results = results.concat(ss.filter((factory) => _match(factory.address.country, searchInput)))
  results = results.concat(ss.filter((factory) => _match(factory.address.city, searchInput)))
  results = results.concat(ss.filter((factory) => _match(factory.address.state_province, searchInput)))
  results = results.concat(ss.filter((factory) => _match(factory.target_id, searchInput)))
  results = results.concat(ss.filter((factory) => _searchCertificateId(factory.certificates, searchInput)))
  results = results.concat(ss.filter((factory) => _searchStandardType(factory.certificates, searchInput)))
  let unique_results = new Set(results)
  vnode.state.factories = [...unique_results];
}

const _searchCertificateId = (certificates, searchInput) =>
  Boolean(certificates.find(cert => _match(cert.id, searchInput)))

const _searchStandardType = (certificates, searchInput) =>
  Boolean(certificates.find(cert => _match(cert.standard_name, searchInput)))

const Certifications = {
  _viewName: 'Certifications',
  oninit: (vnode) => {
    vnode.state.value = ""
    retailerSearchService.loadFactories({ expand: true })
      .then((factories) => {
        console.log(factories)
        vnode.state.factories = factories.data
        vnode.state.searchSpace = factories.data
        vnode.state.loading = false
      })
  },
  view: (vnode) => [
    m('.container',
      [m('.row', m('.col-8.offset-md-2', _searchForm(vnode))),
      m('.row', m('.col-10.offset-md-1.mt-5', m(SearchResults, { factories: vnode.state.factories })))]
    )]
}


module.exports = {
  Certifications
}
