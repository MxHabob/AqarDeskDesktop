pub(crate) struct DefaultContractTemplate {
    pub key: &'static str,
    pub name: &'static str,
    pub legacy_name: &'static str,
    pub lease_type: &'static str,
    pub locale: &'static str,
    pub is_active: bool,
    pub content: &'static str,
}

pub(crate) const DEFAULT_CONTRACT_TEMPLATES: [DefaultContractTemplate; 8] = [
    DefaultContractTemplate {
        key: "residential-lease-ar",
        name: "نموذج عقد إيجار سكني",
        legacy_name: "قالب إيجار سكني (فارغ)",
        lease_type: "residential",
        locale: "ar",
        is_active: false,
        content: r#"# عقد إيجار سكني

رقم العقد: {{contract.number}}  
تاريخ العقد: {{contract.date}}

## أطراف العقد
- المؤجر / ممثل العقار: {{organization.name}}
- المستأجر: {{tenant.full_name}}
- البريد الإلكتروني: {{tenant.email}}
- رقم التواصل: {{tenant.phone}}
- عنوان المستأجر: {{tenant.address}}

## العين المؤجرة
- العقار: {{property.name}} ({{property.code}})
- العنوان: {{property.address}}، {{property.city}}
- الوحدة: {{unit.number}} — {{unit.type}}
- الطابق: {{unit.floor}} · المساحة: {{unit.area}}
- غرف النوم: {{unit.bedrooms}} · دورات المياه: {{unit.bathrooms}}

## مدة الإيجار
تبدأ مدة الإيجار في {{contract.start_date}} وتنتهي في {{contract.end_date}}.

## الأجرة والمبالغ
- قيمة الإيجار المتفق عليها: {{unit.rent}} ({{unit.currency}})
- مبلغ التأمين المتفق عليه: {{contract.deposit}}
- مواعيد السداد وطريقته: [تُستكمل بالاتفاق بين الطرفين]

## تفاصيل وشروط الاتفاق
[يُضاف هنا ما اتفق عليه الطرفان بشأن التسليم والصيانة والمرافق وأي شروط خاصة، ثم يُراجع قبل الاعتماد.]

## إقرار وتوقيع الطرفين
يقر الطرفان بمراجعة البيانات والشروط المدونة أعلاه، وتُستكمل التوقيعات بعد المراجعة والاعتماد.

المؤجر / الممثل: ____________________    التاريخ: __________

المستأجر: __________________________    التاريخ: __________
"#,
    },
    DefaultContractTemplate {
        key: "commercial-lease-ar",
        name: "نموذج عقد إيجار تجاري",
        legacy_name: "قالب إيجار تجاري (فارغ)",
        lease_type: "commercial",
        locale: "ar",
        is_active: false,
        content: r#"# عقد إيجار تجاري

رقم العقد: {{contract.number}}  
تاريخ العقد: {{contract.date}}

## أطراف العقد
- المؤجر / ممثل العقار: {{organization.name}}
- المستأجر / المنشأة: {{tenant.full_name}}
- البريد الإلكتروني: {{tenant.email}}
- رقم التواصل: {{tenant.phone}}
- العنوان: {{tenant.address}}

## الوحدة والغرض من الاستخدام
- العقار: {{property.name}} ({{property.code}})
- العنوان: {{property.address}}، {{property.city}}
- الوحدة التجارية: {{unit.number}} — {{unit.type}}
- الطابق: {{unit.floor}} · المساحة: {{unit.area}}
- الغرض من الاستخدام: [يُحدد النشاط المتفق عليه]

## مدة الإيجار
تبدأ مدة الإيجار في {{contract.start_date}} وتنتهي في {{contract.end_date}}.

## الأجرة والمبالغ
- قيمة الإيجار المتفق عليها: {{unit.rent}} ({{unit.currency}})
- مبلغ التأمين المتفق عليه: {{contract.deposit}}
- جدول السداد والرسوم إن وجدت: [تُستكمل بالاتفاق بين الطرفين]

## شروط النشاط والتسليم
[تُستكمل تفاصيل التسليم والتجهيزات والمرافق وشروط النشاط وأي التزامات خاصة، وتُراجع قبل الاعتماد.]

## إقرار وتوقيع الطرفين
يقر الطرفان بمراجعة بيانات الوحدة والغرض من استخدامها والشروط المدونة أعلاه.

المؤجر / الممثل: ____________________    التاريخ: __________

المستأجر / المفوض: __________________    الصفة: ___________

التوقيع: ___________________________    التاريخ: __________
"#,
    },
    DefaultContractTemplate {
        key: "short-term-lease-ar",
        name: "نموذج عقد إيجار قصير الأجل",
        legacy_name: "قالب إيجار قصير الأجل (فارغ)",
        lease_type: "short_term",
        locale: "ar",
        is_active: false,
        content: r#"# اتفاق إيجار قصير الأجل

رقم الاتفاق: {{contract.number}}  
تاريخ الاتفاق: {{contract.date}}

## بيانات الطرفين
- المؤجر / ممثل العقار: {{organization.name}}
- المستأجر: {{tenant.full_name}}
- البريد الإلكتروني: {{tenant.email}}
- رقم التواصل: {{tenant.phone}}
- عنوان المستأجر: {{tenant.address}}

## مكان الإقامة
- العقار: {{property.name}} ({{property.code}})
- العنوان: {{property.address}}، {{property.city}}
- الوحدة: {{unit.number}} — {{unit.type}}
- الطابق: {{unit.floor}} · المساحة: {{unit.area}}

## فترة الإقامة
من {{contract.start_date}} إلى {{contract.end_date}}.

## المقابل المالي
- قيمة الإيجار المتفق عليها: {{unit.rent}} ({{unit.currency}})
- مبلغ التأمين إن وجد: {{contract.deposit}}
- مواعيد وطريقة السداد: [تُستكمل بالاتفاق بين الطرفين]

## ترتيبات الإقامة والتسليم
[تُستكمل أوقات الدخول والمغادرة، وحالة الوحدة ومحتوياتها، وأي ترتيبات أو شروط خاصة متفق عليها.]

## إقرار وتوقيع الطرفين
يراجع الطرفان بيانات الإقامة والمبالغ والترتيبات قبل التوقيع.

المؤجر / الممثل: ____________________    التاريخ: __________

المستأجر: __________________________    التاريخ: __________
"#,
    },
    DefaultContractTemplate {
        key: "residential-lease-en",
        name: "Residential Lease Agreement",
        legacy_name: "Residential lease (blank)",
        lease_type: "residential",
        locale: "en",
        is_active: false,
        content: r#"# Residential Lease Agreement

Agreement number: {{contract.number}}  
Agreement date: {{contract.date}}

## Parties
- Landlord / property representative: {{organization.name}}
- Tenant: {{tenant.full_name}}
- Email: {{tenant.email}}
- Phone: {{tenant.phone}}
- Tenant address: {{tenant.address}}

## Premises
- Property: {{property.name}} ({{property.code}})
- Address: {{property.address}}, {{property.city}}
- Unit: {{unit.number}} — {{unit.type}}
- Floor: {{unit.floor}} · Area: {{unit.area}}
- Bedrooms: {{unit.bedrooms}} · Bathrooms: {{unit.bathrooms}}

## Lease term
The agreed lease term begins on {{contract.start_date}} and ends on {{contract.end_date}}.

## Rent and amounts
- Agreed rent: {{unit.rent}} ({{unit.currency}})
- Agreed security deposit: {{contract.deposit}}
- Payment schedule and method: [Complete as agreed by the parties]

## Agreed terms
[Add the parties' agreed handover, maintenance, utilities, and any special terms. Review before approval.]

## Acknowledgment and signatures
The parties confirm that they have reviewed the details and terms above. Complete signatures after review and approval.

Landlord / representative: ______________    Date: __________

Tenant: ________________________________    Date: __________
"#,
    },
    DefaultContractTemplate {
        key: "commercial-lease-en",
        name: "Commercial Lease Agreement",
        legacy_name: "Commercial lease (blank)",
        lease_type: "commercial",
        locale: "en",
        is_active: false,
        content: r#"# Commercial Lease Agreement

Agreement number: {{contract.number}}  
Agreement date: {{contract.date}}

## Parties
- Landlord / property representative: {{organization.name}}
- Tenant / business: {{tenant.full_name}}
- Email: {{tenant.email}}
- Phone: {{tenant.phone}}
- Address: {{tenant.address}}

## Premises and permitted use
- Property: {{property.name}} ({{property.code}})
- Address: {{property.address}}, {{property.city}}
- Commercial unit: {{unit.number}} — {{unit.type}}
- Floor: {{unit.floor}} · Area: {{unit.area}}
- Agreed business use: [Specify the parties' agreed activity]

## Lease term
The agreed lease term begins on {{contract.start_date}} and ends on {{contract.end_date}}.

## Rent and amounts
- Agreed rent: {{unit.rent}} ({{unit.currency}})
- Agreed security deposit: {{contract.deposit}}
- Payment schedule and any fees: [Complete as agreed by the parties]

## Handover and business terms
[Complete handover, fit-out, utilities, permitted-use details, and any special obligations. Review before approval.]

## Acknowledgment and signatures
The parties confirm that they have reviewed the premises, permitted use, and terms above.

Landlord / representative: ______________    Date: __________

Tenant / authorized signatory: __________    Title: __________

Signature: _____________________________    Date: __________
"#,
    },
    DefaultContractTemplate {
        key: "short-term-lease-en",
        name: "Short-Term Stay Agreement",
        legacy_name: "Short-term lease (blank)",
        lease_type: "short_term",
        locale: "en",
        is_active: false,
        content: r#"# Short-Term Stay Agreement

Agreement number: {{contract.number}}  
Agreement date: {{contract.date}}

## Parties
- Host / property representative: {{organization.name}}
- Guest: {{tenant.full_name}}
- Email: {{tenant.email}}
- Phone: {{tenant.phone}}
- Guest address: {{tenant.address}}

## Accommodation
- Property: {{property.name}} ({{property.code}})
- Address: {{property.address}}, {{property.city}}
- Unit: {{unit.number}} — {{unit.type}}
- Floor: {{unit.floor}} · Area: {{unit.area}}

## Stay period
The agreed stay begins on {{contract.start_date}} and ends on {{contract.end_date}}.

## Price and amounts
- Agreed rent: {{unit.rent}} ({{unit.currency}})
- Security deposit, if applicable: {{contract.deposit}}
- Payment schedule and method: [Complete as agreed by the parties]

## Handover and stay arrangements
[Complete check-in/check-out times, unit condition and inventory, and any agreed stay arrangements.]

## Acknowledgment and signatures
The parties should review the stay details, amounts, and arrangements before signing.

Host / representative: _________________    Date: __________

Guest: ________________________________    Date: __________
"#,
    },
    DefaultContractTemplate {
        key: "payment-receipt-ar",
        name: "قالب إيصال قبض",
        legacy_name: "",
        lease_type: "receipt",
        locale: "ar",
        is_active: true,
        content: r#"# إيصال قبض

## بيانات الإيصال
- رقم الإيصال: {{payment.receipt_number}}
- تاريخ القبض: {{payment.date}}
- الحالة: {{payment.status}}

## استلمنا من
- المستأجر: {{payment.tenant}}

## تفاصيل المبلغ
- المبلغ: {{payment.amount}} ({{payment.currency}})
- توزيع الدفعة: {{payment.allocations}}
- طريقة الدفع: {{payment.method}}
- المرجع: {{payment.reference}}

## ملاحظات
{{payment.notes}}

تم إصدار هذا الإيصال بواسطة {{organization.name}}.
"#,
    },
    DefaultContractTemplate {
        key: "payment-receipt-en",
        name: "Payment Receipt Template",
        legacy_name: "",
        lease_type: "receipt",
        locale: "en",
        is_active: true,
        content: r#"# Payment Receipt

## Receipt details
- Receipt number: {{payment.receipt_number}}
- Date received: {{payment.date}}
- Status: {{payment.status}}

## Received from
- Tenant: {{payment.tenant}}

## Payment details
- Amount: {{payment.amount}} ({{payment.currency}})
- Allocation: {{payment.allocations}}
- Payment method: {{payment.method}}
- Reference: {{payment.reference}}

## Notes
{{payment.notes}}

Issued by {{organization.name}}.
"#,
    },
];
